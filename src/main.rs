use axum::{
    async_trait,
    extract::{FromRequest, Query, RequestParts, TypedHeader},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Duration;
use clap::Parser;
use headers::{authorization::Bearer, Authorization};
use indoc::indoc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

mod auth;
mod data;
mod error;

/// Handler to return the list of available log types.
async fn log_types() -> &'static str {
    indoc!(
        r#"
        {
        "logtypes": [
            {
            "log_type": "BIND_DNS",
            "description": "BIND DNS Server"
            },
            {
            "log_type": "WINDOWS_DNS",
            "description": "Windows DNS"
            },
            {
            "log_type": "WINDOWS_DHCP",
            "description": "Windows DHCP"
            },
            {
            "log_type": "WINEVTLOG",
            "description": "Windows Event Log"
            }
        ]
        }"#
    )
}

#[derive(Deserialize)]
struct LogType {
    log_type: Option<String>,
}

/// Handler to return the posted logs.
/// This isn't a part of the chronicle API, but is useful for tests
/// to be able to retrieve what has been posted.
async fn logs(log_type: Query<LogType>) -> Json<Vec<data::Log>> {
    let data = data::DATA.lock().unwrap();
    let logs = match log_type.0.log_type {
        Some(log_type) => data
            .iter()
            .filter(|log| log.log_type == log_type)
            .cloned()
            .collect(),
        None => data.to_vec(),
    };

    Json(logs)
}

/// Handler that posts a set of unstructured log entries.
/// To test invalid entries, if a `log_type` of "INVALID" is passed
/// the handler will return a 400 BAD_REQUEST response.
async fn create_unstructured(
    Json(payload): Json<data::UnstructuredLogs>,
    _user: User,
) -> impl IntoResponse {
    if payload.log_type == "INVALID" {
        (StatusCode::BAD_REQUEST, Json(false))
    } else {
        data::add_to_data(payload);
        (StatusCode::CREATED, Json(true))
    }
}

#[derive(Debug, Serialize)]
pub struct TokenPayload {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Handler to retrieve a jwt token.
/// The token request must have been signed by the private key for which we have
/// the public key.
async fn token(public_key: String, body: String) -> Result<Json<TokenPayload>, error::ApiError> {
    for pairs in body.split('&') {
        let mut kv = pairs.split('=');
        let key = kv.next().unwrap();
        let value = percent_encoding::percent_decode_str(kv.next().unwrap())
            .decode_utf8_lossy()
            .to_owned();

        if key == "assertion" {
            let _ = decode::<auth::AuthClaims>(
                &value,
                &DecodingKey::from_rsa_pem(public_key.as_bytes()).unwrap(),
                &Validation::new(Algorithm::RS256),
            )
            .map_err(error::Error::Jwt)?;
        }
    }

    // We only have a single user (user id 1) in our little emulator.
    let token = auth::sign(1);
    let exp = chrono::Utc::now() + Duration::hours(24);

    Ok(Json(TokenPayload {
        expires_in: exp.timestamp(),
        access_token: token,
        token_type: "Bearer".to_string(),
    }))
}

struct User;

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = error::ApiError;

    /// Loads the user from the request. In this dummy app we just check that the
    /// token has been correctly signed as user id 1.
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(error::Error::from)?;

        let claims = auth::verify(bearer.token())?;
        if claims.sub != 1 {
            Err(error::Error::WrongCredentials.into())
        } else {
            Ok(User)
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'p', long)]
    public_key: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let public_key = std::fs::read_to_string(args.public_key).unwrap();

    // build our application with a single route
    let app = Router::new()
        // Fetch the log types
        .route("/v2/logtypes", get(log_types))
        // Token
        .route("/token", post(|body| token(public_key, body)))
        // Post a log entry
        .route(
            "/v2/unstructuredlogentries:batchCreate",
            post(create_unstructured),
        )
        .route("/logs", get(logs));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

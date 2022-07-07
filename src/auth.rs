use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: usize,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(id: usize) -> Self {
        let iat = chrono::Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: id,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn sign(id: usize) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims::new(id),
        &EncodingKey::from_secret("SECRET".as_bytes()),
    )
    .unwrap()
}

pub fn verify(token: &str) -> Result<Claims, error::Error> {
    Ok(jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret("SECRET".as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)?)
}

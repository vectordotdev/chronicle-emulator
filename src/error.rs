use axum::{http::StatusCode, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Header(#[from] axum::extract::rejection::TypedHeaderRejection),
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("password doesn't match")]
    WrongPassword,
}

pub type ApiError = (StatusCode, Json<serde_json::Value>);

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let status = match err {
            Error::WrongCredentials => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let payload = json!({"message": err.to_string()});
        (status, Json(payload))
    }
}

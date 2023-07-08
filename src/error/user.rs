use crate::response::AppErrorResponse;
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User already exists")]
    AlreadyExists,
    #[error("User not found")]
    NotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Error sending mail")]
    MailError,
    #[error("Not verified user")]
    NotVerified,
    #[error("Invalid fingerprint")]
    InvalidFingerprint,
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::MailError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::NotVerified => StatusCode::UNAUTHORIZED,
            UserError::InvalidFingerprint => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

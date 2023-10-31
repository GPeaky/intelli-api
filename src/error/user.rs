use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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
    #[error("Unauthorized user")]
    Unauthorized,
    #[error("Cannot Delete Yourself")]
    AutoDelete,
    #[error("User Already Active")]
    AlreadyActive,
    #[error("User is not active")]
    AlreadyInactive,
    #[error("Invalid Provider")]
    InvalidProvider,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::MailError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::NotVerified => StatusCode::UNAUTHORIZED,
            UserError::Unauthorized => StatusCode::UNAUTHORIZED,
            UserError::AutoDelete => StatusCode::BAD_REQUEST,
            UserError::AlreadyActive => StatusCode::BAD_REQUEST,
            UserError::AlreadyInactive => StatusCode::BAD_REQUEST,
            UserError::InvalidProvider => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

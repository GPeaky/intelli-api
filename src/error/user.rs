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
    #[error("Not verified user")]
    NotVerified,
    #[error("Use google to login")]
    GoogleLogin,
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
    #[error("Using wrong provider")]
    WrongProvider,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::NotVerified => StatusCode::UNAUTHORIZED,
            UserError::GoogleLogin => StatusCode::BAD_REQUEST,
            UserError::Unauthorized => StatusCode::UNAUTHORIZED,
            UserError::AutoDelete => StatusCode::BAD_REQUEST,
            UserError::AlreadyActive => StatusCode::BAD_REQUEST,
            UserError::AlreadyInactive => StatusCode::BAD_REQUEST,
            UserError::InvalidProvider => StatusCode::BAD_REQUEST,
            UserError::WrongProvider => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Missing Bearer token")]
    MissingToken,
    #[error("Token error: {0}")]
    TokenCreationError(String),
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TokenError::InvalidToken => StatusCode::UNAUTHORIZED,
            TokenError::TokenExpired => StatusCode::BAD_REQUEST,
            TokenError::MissingToken => StatusCode::BAD_REQUEST,
            TokenError::TokenCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

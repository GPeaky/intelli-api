use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

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
    #[error("Token not found")]
    TokenNotFound,
    #[error("Invalid token type")]
    InvalidTokenType,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        let code = match self {
            TokenError::InvalidToken => StatusCode::UNAUTHORIZED,
            TokenError::TokenExpired => StatusCode::BAD_REQUEST,
            TokenError::MissingToken => StatusCode::BAD_REQUEST,
            TokenError::TokenCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TokenError::TokenNotFound => StatusCode::NOT_FOUND,
            TokenError::InvalidTokenType => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(code, Some(self.to_string()))
    }
}

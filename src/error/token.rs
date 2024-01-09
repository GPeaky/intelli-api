use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
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
    #[error("Token not found")]
    TokenNotFound,
    #[error("Invalid token type")]
    InvalidTokenType,
}

impl WebResponseError for TokenError {
    fn status_code(&self) -> StatusCode {
        match self {
            TokenError::InvalidToken => StatusCode::UNAUTHORIZED,
            TokenError::TokenExpired => StatusCode::BAD_REQUEST,
            TokenError::MissingToken => StatusCode::BAD_REQUEST,
            TokenError::TokenCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TokenError::TokenNotFound => StatusCode::NOT_FOUND,
            TokenError::InvalidTokenType => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum TokenError {
    InvalidToken,
    MissingToken,
    TokenCreationError,
    InvalidTokenType,
}

impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl TokenError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            TokenError::InvalidToken => StatusCode::UNAUTHORIZED,
            TokenError::MissingToken => StatusCode::BAD_REQUEST,
            TokenError::TokenCreationError => StatusCode::INTERNAL_SERVER_ERROR,
            TokenError::InvalidTokenType => StatusCode::BAD_REQUEST,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            TokenError::InvalidToken => "Invalid token",
            TokenError::MissingToken => "Missing Bearer token",
            TokenError::TokenCreationError => "Token Creation Error",
            TokenError::InvalidTokenType => "Invalid token type",
        }
    }
}

impl From<TokenError> for AppError {
    fn from(e: TokenError) -> Self {
        AppError::Token(e)
    }
}

impl WebResponseError for TokenError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/plain; charset=utf-8")
            .body(self.error_message())
    }
}

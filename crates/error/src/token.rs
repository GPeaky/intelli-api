use ntex::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        StatusCode,
    },
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum TokenError {
    InvalidToken,
    MissingToken,
    ExpiredToken,
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
            TokenError::ExpiredToken => StatusCode::BAD_REQUEST,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            TokenError::InvalidToken => "Invalid token",
            TokenError::MissingToken => "Missing token",
            TokenError::ExpiredToken => "Expired token",
        }
    }
}

impl From<TokenError> for AppError {
    #[inline]
    fn from(e: TokenError) -> Self {
        AppError::Token(e)
    }
}

// Added for middlewares
impl WebResponseError for TokenError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )
            .body(self.error_message())
    }
}

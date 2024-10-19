use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum CommonError {
    ValidationFailed,
    InternalServerError,
    HashingFailed,
    NotValidUpdate,
    RateLimited,
    UpdateLimit,
}

impl CommonError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            CommonError::ValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::HashingFailed => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::NotValidUpdate => StatusCode::BAD_REQUEST,
            CommonError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            CommonError::UpdateLimit => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            CommonError::ValidationFailed => "Data validation failed",
            CommonError::InternalServerError => "Internal server error",
            CommonError::HashingFailed => "Hashing Failed",
            CommonError::NotValidUpdate => "Not valid Update",
            CommonError::RateLimited => "Rate limited",
            CommonError::UpdateLimit => "Update limit exceeded",
        }
    }
}

impl std::error::Error for CommonError {}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl From<CommonError> for AppError {
    #[inline]
    fn from(e: CommonError) -> Self {
        AppError::Common(e)
    }
}

impl WebResponseError for CommonError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/plain; charset=utf-8")
            .body(self.error_message())
    }
}

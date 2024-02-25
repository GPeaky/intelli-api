use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum CommonError {
    ValidationFailed,
    InternalServerError,
    SendMail,
    HashingFailed,
    NotValidUpdate,
    InvalidUsedFeature(&'static str),
}

impl CommonError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            CommonError::ValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::SendMail => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::HashingFailed => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::NotValidUpdate => StatusCode::BAD_REQUEST,
            CommonError::InvalidUsedFeature(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            CommonError::ValidationFailed => "Form validation failed",
            CommonError::InternalServerError => "Internal Server Error",
            CommonError::SendMail => "Error to send mail",
            CommonError::HashingFailed => "Hashing Failed",
            CommonError::NotValidUpdate => "Not valid Update",
            CommonError::InvalidUsedFeature(feature) => feature,
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

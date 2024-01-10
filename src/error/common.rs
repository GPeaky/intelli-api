use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Form validation failed")]
    ValidationFailed,
    #[error("Not Ports Available")]
    NotPortsAvailable,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Not valid Update")]
    NotValidUpdate,
    #[error("Invalid Used Feature {0}")]
    InvalidUsedFeature(String),
}

impl WebResponseError for CommonError {
    fn status_code(&self) -> StatusCode {
        match self {
            CommonError::ValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::NotPortsAvailable => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::NotValidUpdate => StatusCode::BAD_REQUEST,
            CommonError::InvalidUsedFeature(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

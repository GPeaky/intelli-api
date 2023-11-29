use ntex::{http::StatusCode, web};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Form validation failed")]
    FormValidationFailed,
    #[error("Not Ports Available")]
    NotPortsAvailable,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Not valid Update")]
    NotValidUpdate,
}

impl web::error::WebResponseError for CommonError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            CommonError::FormValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::NotPortsAvailable => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::NotValidUpdate => StatusCode::BAD_REQUEST,
        }
    }
}

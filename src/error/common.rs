use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Form validation failed")]
    FormValidationFailed,
    #[error("Not Ports Available")]
    NotPortsAvailable,
    #[error("Mail Server Error")]
    MailServerError,
}

impl IntoResponse for CommonError {
    fn into_response(self) -> Response {
        let status_code = match self {
            CommonError::FormValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::NotPortsAvailable => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::MailServerError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

use crate::response::AppErrorResponse;
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Form validation failed")]
    FormValidationFailed,
    #[error("Not Ports Available")]
    NotPortsAvailable,
}

impl IntoResponse for CommonError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            CommonError::FormValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::NotPortsAvailable => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

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

impl IntoResponse for CommonError {
    fn into_response(self) -> Response {
        let code = match self {
            CommonError::ValidationFailed => StatusCode::BAD_REQUEST,
            CommonError::NotPortsAvailable => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::NotValidUpdate => StatusCode::BAD_REQUEST,
            CommonError::InvalidUsedFeature(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(code, Some(self.to_string()))
    }
}

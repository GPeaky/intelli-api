use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("Socket not found")]
    NotFound,
    #[error("Socket already exists")]
    AlreadyExists,
    #[error("Socket is not active")]
    NotActive,
    // #[error("Rule Already Exists")]
    // RuleAlreadyExists,
    // #[error("Command failed")]
    // CommandFailed,
}

impl IntoResponse for SocketError {
    fn into_response(self) -> Response {
        let status_code = match self {
            SocketError::NotFound => StatusCode::BAD_REQUEST,
            SocketError::AlreadyExists => StatusCode::CONFLICT,
            SocketError::NotActive => StatusCode::NOT_FOUND,
            // SocketError::RuleAlreadyExists => StatusCode::CONFLICT,
            // SocketError::CommandFailed => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

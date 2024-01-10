use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("Socket not found")]
    NotFound,
    #[error("Socket already exists")]
    AlreadyExists,
    #[error("Socket is not active")]
    NotActive,
    #[error("Failed to send message")]
    FailedToSendMessage,
}

impl IntoResponse for SocketError {
    fn into_response(self) -> Response {
        let code = match self {
            SocketError::NotFound => StatusCode::BAD_REQUEST,
            SocketError::AlreadyExists => StatusCode::CONFLICT,
            SocketError::NotActive => StatusCode::NOT_FOUND,
            SocketError::FailedToSendMessage => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(code, Some(self.to_string()))
    }
}

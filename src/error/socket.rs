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
}

impl IntoResponse for SocketError {
    fn into_response(self) -> Response {
        let status_code = match self {
            SocketError::NotFound => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

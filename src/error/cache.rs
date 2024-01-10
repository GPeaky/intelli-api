use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Error deserializing entity from cache")]
    Deserialize,
    #[error("Error serializing entity to cache")]
    Serialize,
}

impl IntoResponse for CacheError {
    fn into_response(self) -> Response {
        AppErrorResponse::send(StatusCode::INTERNAL_SERVER_ERROR, None)
    }
}

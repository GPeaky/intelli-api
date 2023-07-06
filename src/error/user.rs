use crate::response::AppErrorResponse;
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User already exists")]
    AlreadyExists,
}

impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

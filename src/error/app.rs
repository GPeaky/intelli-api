use super::user::UserError;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    User(#[from] UserError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::User(err) => err.into_response(),
        }
    }
}

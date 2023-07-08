use crate::response::AppErrorResponse;
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChampionshipError {
    #[error("Championship already exists")]
    AlreadyExists,
    // #[error("Championship not found")]
    // NotFound,
}

impl IntoResponse for ChampionshipError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            ChampionshipError::AlreadyExists => StatusCode::CONFLICT,
            // ChampionshipError::NotFound => StatusCode::NOT_FOUND,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

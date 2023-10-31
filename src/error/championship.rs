use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[allow(unused)]
#[derive(Debug, Error)]
pub enum ChampionshipError {
    #[error("Championship already exists")]
    AlreadyExists,
    #[error("Not Championships")]
    NotChampionships,
    #[error("Championship not found")]
    NotFound,
    #[error("Championship limit reached")]
    LimitReached,
}

impl IntoResponse for ChampionshipError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ChampionshipError::AlreadyExists => StatusCode::CONFLICT,
            ChampionshipError::NotChampionships => StatusCode::NOT_FOUND,
            ChampionshipError::NotFound => StatusCode::NOT_FOUND,
            ChampionshipError::LimitReached => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(status_code, Some(self.to_string()))
    }
}

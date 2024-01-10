use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

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
    #[error("Not Owner of Championship")]
    NotOwner,
    #[error("Cannot remove owner of Championship")]
    CannotRemoveOwner,
    #[error("Interval update time not reached")]
    IntervalNotReached,
}

impl IntoResponse for ChampionshipError {
    fn into_response(self) -> Response {
        let code = match self {
            ChampionshipError::AlreadyExists => StatusCode::CONFLICT,
            ChampionshipError::NotChampionships => StatusCode::NOT_FOUND,
            ChampionshipError::NotFound => StatusCode::NOT_FOUND,
            ChampionshipError::LimitReached => StatusCode::BAD_REQUEST,
            ChampionshipError::NotOwner => StatusCode::UNAUTHORIZED,
            ChampionshipError::CannotRemoveOwner => StatusCode::BAD_REQUEST,
            ChampionshipError::IntervalNotReached => StatusCode::BAD_REQUEST,
        };

        AppErrorResponse::send(code, Some(self.to_string()))
    }
}

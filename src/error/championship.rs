use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
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
    #[error("Not Owner of Championship")]
    NotOwner,
    #[error("Cannot remove owner of Championship")]
    CannotRemoveOwner,
    #[error("Interval update time not reached")]
    IntervalNotReached,
}

impl WebResponseError for ChampionshipError {
    fn status_code(&self) -> StatusCode {
        match self {
            ChampionshipError::AlreadyExists => StatusCode::CONFLICT,
            ChampionshipError::NotChampionships => StatusCode::NOT_FOUND,
            ChampionshipError::NotFound => StatusCode::NOT_FOUND,
            ChampionshipError::LimitReached => StatusCode::BAD_REQUEST,
            ChampionshipError::NotOwner => StatusCode::UNAUTHORIZED,
            ChampionshipError::CannotRemoveOwner => StatusCode::BAD_REQUEST,
            ChampionshipError::IntervalNotReached => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

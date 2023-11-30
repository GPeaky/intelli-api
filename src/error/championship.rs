use ntex::{http::StatusCode, web};
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

impl web::error::WebResponseError for ChampionshipError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }

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
}

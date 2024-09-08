use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpResponse,
};

use crate::{
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::ChampionshipId,
};

#[inline(always)]
pub async fn start(state: State<AppState>, path: Path<ChampionshipId>) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repo.find(path.0).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state
        .f1_svc
        .start(championship.port, championship.id)
        .await?;

    Ok(HttpResponse::Created().finish())
}

#[inline(always)]
pub async fn status(state: State<AppState>, path: Path<ChampionshipId>) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    if state.championship_repo.find(path.0).await?.is_none() {
        Err(ChampionshipError::NotFound)?
    }

    let service_status = state.f1_svc.service_status(&path.0);

    Ok(HttpResponse::Ok().json(&service_status))
}

#[inline(always)]
pub async fn stop(state: State<AppState>, path: Path<ChampionshipId>) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state.f1_svc.stop(&path.0).await?;

    Ok(HttpResponse::Ok().finish())
}

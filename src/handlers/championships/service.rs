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
pub async fn active_services(state: State<AppState>) -> AppResult<HttpResponse> {
    let services = state.f1_svc.services();
    Ok(HttpResponse::Ok().json(&services))
}

#[inline(always)]
pub async fn start_service(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
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
pub async fn service_status(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    // Only used to check if the championship requested exist
    if state.championship_repo.find(path.0).await?.is_none() {
        Err(ChampionshipError::NotFound)?
    }

    let service_status = state.f1_svc.service_status(&path.0);

    Ok(HttpResponse::Ok().json(&service_status))
}

#[inline(always)]
pub async fn stop_service(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state.f1_svc.stop(&path.0).await?;

    Ok(HttpResponse::Ok().finish())
}

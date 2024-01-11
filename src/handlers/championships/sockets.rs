use std::sync::Arc;

use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpResponse, Responder,
};

use crate::{
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{ChampionshipIdPath, SocketStatus},
};

use super::counter::get;

#[inline(always)]
pub async fn active_sockets(state: State<AppState>) -> AppResult<impl Responder> {
    let sockets = state.f123_service.get_active_socket_ids().await;
    Ok(HttpResponse::Ok().json(&sockets))
}

#[inline(always)]
pub async fn start_socket(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state
        .f123_service
        .setup_championship_listening_socket(championship.port, Arc::new(championship.id))
        .await?;

    Ok(HttpResponse::Created())
}

#[inline(always)]
pub async fn socket_status(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    let mut num_connections = 0;
    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    if socket_active {
        if let Some(count) = get(path.id) {
            num_connections = count;
        };
    }

    let socket_status = SocketStatus {
        active: socket_active,
        connections: num_connections,
    };

    Ok(HttpResponse::Ok().json(&socket_status))
}

#[inline(always)]
pub async fn stop_socket(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state.f123_service.stop_socket(path.id).await?;

    Ok(HttpResponse::Ok())
}

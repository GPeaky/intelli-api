use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;

use crate::{
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{ChampionshipIdPath, SocketStatus},
};

// use super::counter::get;

#[inline(always)]
pub async fn active_sockets(state: State<AppState>) -> AppResult<Json<Vec<i32>>> {
    let sockets = state.f123_service.get_active_socket_ids().await;
    Ok(Json(sockets))
}

#[inline(always)]
pub async fn start_socket(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Response> {
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

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub async fn socket_status(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Json<SocketStatus>> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    let num_connections = 0;
    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    // if socket_active {
    //     if let Some(count) = get(path.id) {
    //         num_connections = count;
    //     };
    // }

    let socket_status = SocketStatus {
        active: socket_active,
        connections: num_connections,
    };

    Ok(Json(socket_status))
}

#[inline(always)]
pub async fn stop_socket(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Response> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state.f123_service.stop_socket(path.id).await?;

    Ok(StatusCode::OK.into_response())
}

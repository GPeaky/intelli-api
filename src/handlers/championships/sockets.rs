use crate::{
    dtos::SocketStatus,
    error::{AppResult, ChampionshipError},
    handlers::championships::websocket_active_connections,
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use std::sync::Arc;

#[inline(always)]
pub async fn active_sockets(State(state): State<UserState>) -> AppResult<Json<Vec<i32>>> {
    let sockets = state.f123_service.get_active_socket_ids().await;
    Ok(Json(sockets))
}

#[inline(always)]
pub async fn start_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<i32>,
) -> AppResult<Response> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
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
    State(state): State<UserState>,
    Path(championship_id): Path<i32>,
) -> AppResult<Json<SocketStatus>> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    let mut num_connections = 0;
    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    if socket_active {
        num_connections = websocket_active_connections(championship_id).await;
    }

    Ok(Json(SocketStatus {
        active: socket_active,
        connections: num_connections,
    }))
}

#[inline(always)]
pub async fn stop_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<i32>,
) -> AppResult<Response> {
    state.f123_service.stop_socket(championship_id).await?;

    Ok(StatusCode::OK.into_response())
}

use crate::{
    error::{AppResult, UserError},
    states::SafeUserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use std::sync::Arc;

#[inline(always)]
pub async fn active_sockets(State(state): State<SafeUserState>) -> AppResult<Json<Vec<u32>>> {
    let sockets = state.f123_service.active_sockets().await;
    Ok(Json(sockets))
}

#[inline(always)]
pub async fn start_socket(
    State(state): State<SafeUserState>,
    Path(championship_id): Path<u32>,
) -> AppResult<Response> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(UserError::ChampionshipNotFound)?
    };

    state
        .f123_service
        .new_socket(championship.port, Arc::new(championship.id))
        .await?;

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub async fn stop_socket(
    State(state): State<SafeUserState>,
    Path(championship_id): Path<u32>,
) -> AppResult<Response> {
    state.f123_service.stop_socket(championship_id).await?;

    Ok(StatusCode::OK.into_response())
}

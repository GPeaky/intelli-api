use crate::{error::AppResult, states::UserState};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use std::sync::Arc;

#[inline(always)]
pub async fn active_sockets(State(state): State<UserState>) -> AppResult<Json<Vec<String>>> {
    let sockets = state.f123_service.active_sockets().await?;
    Ok(Json(sockets))
}

#[inline(always)]
pub async fn start_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<String>,
) -> AppResult<Response> {
    let championship = state.championship_repository.find(&championship_id).await?;

    state
        .f123_service
        .new_socket(championship.port, Arc::new(championship.id))
        .await?;

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub async fn stop_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<String>,
) -> AppResult<Response> {
    state.f123_service.stop_socket(championship_id).await?;

    Ok(StatusCode::OK.into_response())
}

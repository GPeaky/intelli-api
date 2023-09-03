use crate::{
    entity::Championship,
    error::{AppResult, ChampionshipError},
    states::SafeUserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use scylla::cql_to_rust::FromRowError;

#[inline(always)]
pub async fn user_championships(
    State(state): State<SafeUserState>,
    Path(user_id): Path<i32>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state
        .championship_service
        .user_championships(&user_id)
        .await?;

    let championships = championships
        .into_iter()
        .collect::<Result<Vec<Championship>, FromRowError>>()
        .map_err(|_| ChampionshipError::NotChampionships)?;

    Ok(Json(championships))
}

#[inline(always)]
pub async fn delete_championship(
    State(state): State<SafeUserState>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    let championship = state.championship_repository.find(&id).await?;

    state
        .championship_service
        .delete_championship(&championship.id)
        .await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Update a championship by id
#[inline(always)]
pub async fn update_championship() {
    todo!("Update a championship by id")
}

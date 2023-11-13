use crate::{
    entity::Championship,
    error::{AppResult, ChampionshipError},
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;

#[inline(always)]
pub async fn user_championships(
    State(state): State<UserState>,
    Path(user_id): Path<i32>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state.championship_repository.find_all(&user_id).await?;

    Ok(Json(championships))
}

#[inline(always)]
pub async fn delete_championship(
    State(state): State<UserState>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    let Some(championship) = state.championship_repository.find(&id).await? else {
        Err(ChampionshipError::NotFound)?
    };

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

use crate::{entity::Championship, error::AppResult, states::UserState};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use scylla::cql_to_rust::FromRowError;

// TODO: Return a list of all championships for the user
#[inline(always)]
pub async fn user_championships(
    State(state): State<UserState>,
    Path(user_id): Path<i32>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state
        .championship_service
        .user_championships(&user_id)
        .await?;

    let championships = championships
        .into_iter()
        .collect::<Result<Vec<Championship>, FromRowError>>()
        .unwrap();

    Ok(Json(championships))
}

// TODO: Delete a championship by id
#[inline(always)]
pub async fn delete_championship(
    State(state): State<UserState>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    state.championship_service.delete_championship(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Update a championship by id
#[inline(always)]
pub async fn update_championship() {
    todo!("Update a championship by id")
}

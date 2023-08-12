use crate::{
    dtos::CreateChampionshipDto,
    entity::{Championship, User},
    error::{AppResult, CommonError},
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use garde::Validate;
use hyper::StatusCode;
use scylla::cql_to_rust::FromRowError;
pub(crate) use sockets::*;
pub(crate) use web_socket::*;

mod sockets;
mod web_socket;

#[inline(always)]
pub async fn create_championship(
    Extension(user): Extension<User>,
    State(mut state): State<UserState>,
    Form(form): Form<CreateChampionshipDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    state
        .championship_service
        .create_championship(form, &user.id)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn get_championship(
    State(state): State<UserState>,
    Path(championship_id): Path<i32>,
) -> AppResult<Json<Championship>> {
    let championship = state.championship_repository.find(&championship_id).await?;

    Ok(Json(championship))
}

#[inline(always)]
pub async fn all_championships(
    State(state): State<UserState>,
    Extension(user): Extension<User>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state
        .championship_repository
        .find_all(&user.id)
        .await?
        .collect::<Vec<Result<Championship, FromRowError>>>();

    let championships = championships
        .into_iter()
        .collect::<Result<Vec<Championship>, FromRowError>>()
        .unwrap();

    Ok(Json(championships))
}

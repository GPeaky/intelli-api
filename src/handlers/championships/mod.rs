use crate::{
    dtos::CreateChampionshipDto,
    entity::{Championship, User},
    error::{AppResult, ChampionshipError, CommonError},
    states::SafeUserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use garde::Validate;
use hyper::StatusCode;

pub(crate) use admin::*;
pub(crate) use sockets::*;
pub(crate) use web_socket::*;

mod admin;
mod sockets;
mod web_socket;

const MAXIMUM_CHAMPIONSHIPS: usize = 3;

#[inline(always)]
pub async fn create_championship(
    Extension(user): Extension<User>,
    State(state): State<SafeUserState>,
    Form(form): Form<CreateChampionshipDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let championships = state.championship_repository.find_all(&user.id).await?;

    if championships.len().gt(&MAXIMUM_CHAMPIONSHIPS) {
        Err(ChampionshipError::LimitReached)?;
    }

    state
        .championship_service
        .create_championship(form, &user.id)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn get_championship(
    State(state): State<SafeUserState>,
    Path(championship_id): Path<u32>,
) -> AppResult<Json<Championship>> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(Json(championship))
}

#[inline(always)]
pub async fn all_championships(
    State(state): State<SafeUserState>,
    Extension(user): Extension<User>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(Json(championships))
}

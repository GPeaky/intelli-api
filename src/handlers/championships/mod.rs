use axum::{
    extract::{Extension, Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;

pub(crate) use admin::*;
// pub(crate) use socket::*;
pub(crate) use sockets::*;

use crate::{
    entity::{Championship, Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{
        AddUser, ChampionshipAndUserIdPath, ChampionshipIdPath, CreateChampionshipDto,
        UpdateChampionship,
    },
};

mod admin;
// mod socket;
mod sockets;

#[inline(always)]
pub async fn create_championship(
    state: State<AppState>,
    user: Extension<UserExtension>,
    Form(form): Form<CreateChampionshipDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let championships_len = state
        .championship_repository
        .championship_len(&user.id)
        .await?;

    match user.role {
        Role::Free => {
            if championships_len >= 1 {
                Err(ChampionshipError::LimitReached)?
            }
        }

        Role::Premium => {
            if championships_len >= 3 {
                Err(ChampionshipError::LimitReached)?
            }
        }

        Role::Business => {
            if championships_len >= 14 {
                Err(ChampionshipError::LimitReached)?
            }
        }

        Role::Admin => {}
    }

    state.championship_service.create(form, &user.id).await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn update(
    state: State<AppState>,
    user: Extension<UserExtension>,
    form: Form<UpdateChampionship>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() || path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state
        .championship_service
        .update(&path.id, &user.id, &form)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn add_user(
    state: State<AppState>,
    user: Extension<UserExtension>,
    form: Form<AddUser>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() || path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state
        .championship_service
        .add_user(&path.id, &user.id, &form.email)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn remove_user(
    state: State<AppState>,
    user: Extension<UserExtension>,
    path: Path<ChampionshipAndUserIdPath>,
) -> AppResult<Response> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state
        .championship_service
        .remove_user(&path.id, &user.id, &path.user_id)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn get_championship(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Json<Championship>> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(Json(championship))
}

#[inline(always)]
pub async fn all_championships(
    state: State<AppState>,
    user: Extension<UserExtension>,
) -> AppResult<Json<Vec<Championship>>> {
    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(Json(championships))
}

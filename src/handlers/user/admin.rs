use crate::{
    entity::UserExtension,
    error::{AppResult, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension,
};
use hyper::StatusCode;

// TODO: Add admin user handlers
#[inline(always)]
pub async fn delete_user(
    State(state): State<UserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Response> {
    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Disable a user by id
#[inline(always)]
pub async fn disable_user(
    State(state): State<UserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Response> {
    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user.active.eq(&false) {
        Err(UserError::AlreadyInactive)?
    }

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn enable_user(
    State(state): State<UserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Response> {
    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user.active.eq(&true) {
        Err(UserError::AlreadyActive)?
    }

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate_user(&path_user.id).await?;
    Ok(StatusCode::OK.into_response())
}

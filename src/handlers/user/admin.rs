use crate::{
    entity::User,
    error::{AppResult, UserError},
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
    Path(id): Path<i32>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    if id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Disable a user by id
#[inline(always)]
pub async fn disable_user(
    State(state): State<UserState>,
    Path(id): Path<i32>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    if id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Enable a user by id
#[inline(always)]
pub async fn enable_user(
    State(state): State<UserState>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    state.user_service.activate_user(&id).await?;
    Ok(StatusCode::OK.into_response())
}

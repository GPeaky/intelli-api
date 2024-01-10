use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use garde::Validate;

use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
    structs::UserIdPath,
};

#[inline(always)]
pub async fn delete_user(
    state: State<AppState>,
    user: Extension<UserExtension>,
    path: Path<UserIdPath>,
) -> AppResult<Response> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    if state.user_repository.find(&path.id).await?.is_none() {
        Err(UserError::NotFound)?
    };

    if path.id == user.id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete(&path.id).await?;
    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn disable_user(
    state: State<AppState>,
    user: Extension<UserExtension>,
    path: Path<UserIdPath>,
) -> AppResult<Response> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repository.status(&path.id).await? else {
        Err(UserError::NotFound)?
    };

    if !path_user_active {
        Err(UserError::AlreadyInactive)?
    }

    if path.id == user.id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate(&path.id).await?;
    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn enable_user(
    state: State<AppState>,
    user: Extension<UserExtension>,
    path: Path<UserIdPath>,
) -> AppResult<Response> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repository.status(&path.id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user_active {
        Err(UserError::AlreadyActive)?
    }

    if path.id == user.id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate(&path.id).await?;
    Ok(StatusCode::OK.into_response())
}

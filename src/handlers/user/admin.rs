use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpRequest, HttpResponse, Responder,
};

use crate::{
    dtos::UserIdPath,
    entity::UserExtension,
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
};

#[inline(always)]
pub async fn delete_user(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    if state.user_repository.find(&path.id).await?.is_none() {
        Err(UserError::NotFound)?
    };

    if path.id == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete(&path.id).await?;
    Ok(HttpResponse::Ok())
}

#[inline(always)]
pub async fn disable_user(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repository.status(&path.id).await? else {
        Err(UserError::NotFound)?
    };

    if !path_user_active {
        Err(UserError::AlreadyInactive)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    if path.id == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate(&path.id).await?;
    Ok(HttpResponse::Ok())
}

#[inline(always)]
pub async fn enable_user(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserIdPath>,
) -> AppResult<impl Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repository.status(&path.id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user_active {
        Err(UserError::AlreadyActive)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    if path.id == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate(&path.id).await?;
    Ok(HttpResponse::Ok())
}

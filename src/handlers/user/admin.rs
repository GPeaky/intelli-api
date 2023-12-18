use crate::dtos::UserIdPath;
use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
};
use garde::Validate;
use ntex::web;

#[inline(always)]
pub async fn delete_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    path: web::types::Path<UserIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    let Some(_) = state.user_repository.find(&path.id).await? else {
        Err(UserError::NotFound)?
    };

    if path.id == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete(&path.id).await?;
    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn disable_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    path: web::types::Path<UserIdPath>,
) -> AppResult<impl web::Responder> {
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
    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn enable_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    path: web::types::Path<UserIdPath>,
) -> AppResult<impl web::Responder> {
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
    Ok(web::HttpResponse::Ok())
}

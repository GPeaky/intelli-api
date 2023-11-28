use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
};
use ntex::web;

#[inline(always)]
pub async fn delete_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .clone();

    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete(&id).await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn disable_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .clone();

    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if !path_user.active {
        Err(UserError::AlreadyInactive)?
    }

    if path_user.id == user.id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate(&id).await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn enable_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(path_user) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if path_user.active.eq(&true) {
        Err(UserError::AlreadyActive)?
    }

    let user = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .clone();

    if path_user.id == user.id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate(&path_user.id).await?;
    Ok(web::HttpResponse::Ok())
}

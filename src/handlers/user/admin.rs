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
    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    let Some(_) = state.user_repository.find(&id).await? else {
        Err(UserError::NotFound)?
    };

    if *id == user_id {
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
    let Some(path_user_active) = state.user_repository.status(&id).await? else {
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

    if *id == user_id {
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
    let Some(path_user_active) = state.user_repository.status(&id).await? else {
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

    if *id == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate(&id).await?;
    Ok(web::HttpResponse::Ok())
}

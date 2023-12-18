mod admin;
mod socket;
mod sockets;

use crate::dtos::{ChampionshipAndUserIdPath, ChampionshipIdPath};
use crate::{
    dtos::{AddUser, CreateChampionshipDto, UpdateChampionship},
    entity::{Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
};
pub(crate) use admin::*;
use garde::Validate;
use ntex::web;
pub(crate) use socket::*;
pub(crate) use sockets::*;

#[inline(always)]
pub async fn create_championship(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    form: web::types::Form<CreateChampionshipDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

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

    state
        .championship_service
        .create(form.into_inner(), &user.id)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn update(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    form: web::types::Form<UpdateChampionship>,
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() || path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    state
        .championship_service
        .update(&path.id, &user_id, &form)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn add_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    form: web::types::Form<AddUser>,
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() || path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    state
        .championship_service
        .add_user(&path.id, &user_id, &form.email)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn remove_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    path: web::types::Path<ChampionshipAndUserIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    state
        .championship_service
        .remove_user(&path.id, &user_id, &path.user_id)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn get_championship(
    state: web::types::State<AppState>,
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(web::HttpResponse::Ok().json(&championship))
}

#[inline(always)]
pub async fn all_championships(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
) -> AppResult<impl web::Responder> {
    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    let championships = state.championship_repository.find_all(&user_id).await?;

    Ok(web::HttpResponse::Ok().json(&championships))
}

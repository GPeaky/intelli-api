use crate::{
    dtos::{AddUser, CreateChampionshipDto, UpdateChampionship},
    entity::{Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
};
use garde::Validate;
use ntex::web;

pub(crate) use admin::*;
pub(crate) use socket::*;
pub(crate) use sockets::*;

mod admin;
mod socket;
mod sockets;

#[inline(always)]
pub async fn create_championship(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    form: web::types::Form<CreateChampionshipDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    {
        let championships_len = state
            .championship_repository
            .user_champions_len(&user.id)
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
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        Err(CommonError::FormValidationFailed)?
    }

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    state
        .championship_service
        .update(&championship_id, &user.id, &form)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn add_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
    form: web::types::Form<AddUser>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        Err(CommonError::FormValidationFailed)?
    }

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    state
        .championship_service
        .add_user(&championship_id, &user.id, &form.email)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn remove_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    ids: web::types::Path<(i32, i32)>,
) -> AppResult<impl web::Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    state
        .championship_service
        .remove_user(&ids.0, &user.id, &ids.1)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn get_championship(
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(web::HttpResponse::Ok().json(&championship))
}

#[inline(always)]
pub async fn all_championships(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
) -> AppResult<impl web::Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(web::HttpResponse::Ok().json(&championships))
}

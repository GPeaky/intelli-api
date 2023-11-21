use crate::{
    dtos::CreateChampionshipDto,
    entity::{Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
};
use garde::Validate;
use ntex::web;

pub(crate) use admin::*;
pub(crate) use sockets::*;
pub(crate) use web_socket::*;

mod admin;
mod sockets;
mod web_socket;

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
        .ok_or(CommonError::InternalServerError)?
        .clone();

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
        .create_championship(form.into_inner(), &user.id)
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
        .ok_or(CommonError::InternalServerError)?
        .clone();

    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(web::HttpResponse::Ok().json(&championships))
}

use garde::Validate;
use ntex::web::{
    types::{Form, Path, State},
    HttpRequest, HttpResponse,
};

pub(crate) use admin::*;
pub(crate) use service::*;
pub(crate) use stream::*;

use crate::{
    entity::{Role, UserExtension},
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{
        ChampionshipAndUserId, ChampionshipCreationData, ChampionshipId, ChampionshipUpdateData,
        ChampionshipUserAddForm,
    },
};

mod admin;
mod service;
mod stream;

#[inline(always)]
pub async fn create_championship(
    req: HttpRequest,
    state: State<AppState>,
    Form(form): Form<ChampionshipCreationData>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user = req.user()?;
    let championships_len = state.championship_repo.championship_len(user.id).await?;

    match user.role {
        Role::User => {
            if championships_len >= 1 {
                Err(ChampionshipError::LimitReached)?
            }
        }

        Role::Premium => {
            if championships_len >= 3 {
                Err(ChampionshipError::LimitReached)?
            }
        }

        Role::Admin => {}
    }

    state.championship_svc.create(form, user.id).await?;
    Ok(HttpResponse::Created().finish())
}

#[inline(always)]
pub async fn update(
    req: HttpRequest,
    state: State<AppState>,
    form: Form<ChampionshipUpdateData>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() || path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req.user_id()?;
    state
        .championship_svc
        .update(path.0, user_id, &form)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[inline(always)]
pub async fn add_user(
    req: HttpRequest,
    state: State<AppState>,
    Form(form): Form<ChampionshipUserAddForm>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() || path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req.user_id()?;
    state
        .championship_svc
        .add_user(path.0, user_id, form)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[inline(always)]
pub async fn remove_user(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<ChampionshipAndUserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req.user_id()?;
    state
        .championship_svc
        .remove_user(path.championship_id, user_id, path.user_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[inline(always)]
pub async fn get_championship(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repo.find(path.0).await? else {
        Err(ChampionshipError::NotFound)?
    };

    Ok(HttpResponse::Ok().json(&championship))
}

#[inline(always)]
pub async fn all_championships(
    req: HttpRequest,
    state: State<AppState>,
) -> AppResult<HttpResponse> {
    let user_id = req.user_id()?;
    let championships = state.championship_repo.find_all(user_id).await?;

    Ok(HttpResponse::Ok().json(&championships))
}

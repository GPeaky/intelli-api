use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpRequest, HttpResponse,
};

use entities::UserExtension;
use error::{AppResult, CommonError, UserError};
use intelli_core::services::{UserAdminServiceOperations, UserServiceOperations};
use structs::UserId;

use crate::states::AppState;

#[inline]
pub async fn user_championships(
    state: State<AppState>,
    path: Path<UserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let championships = state.user_repo.championships(path.0).await?;
    Ok(HttpResponse::Ok().json(&championships))
}

#[inline]
pub async fn remove_user(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let user_id = req.user_id()?;

    if state.user_repo.find(path.0).await?.is_none() {
        Err(UserError::NotFound)?
    };

    if path.0 == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_svc.delete(path.0).await?;
    Ok(HttpResponse::Ok().finish())
}

#[inline]
pub async fn deactivate_user_account(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repo.status(path.0).await? else {
        Err(UserError::NotFound)?
    };

    if !path_user_active {
        Err(UserError::AlreadyInactive)?
    }

    let user_id = req.user_id()?;

    if path.0 == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_svc.admin_deactivate(path.0).await?;
    Ok(HttpResponse::Ok().finish())
}

#[inline]
pub async fn reactivate_user_account(
    req: HttpRequest,
    state: State<AppState>,
    path: Path<UserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(path_user_active) = state.user_repo.status(path.0).await? else {
        Err(UserError::NotFound)?
    };

    if path_user_active {
        Err(UserError::AlreadyActive)?
    }

    let user_id = req.user_id()?;

    if path.0 == user_id {
        Err(UserError::AutoDelete)?
    }

    state.user_svc.admin_activate(path.0).await?;
    Ok(HttpResponse::Ok().finish())
}

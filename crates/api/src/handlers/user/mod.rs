pub(crate) mod admin;

use garde::Validate;
use ntex::web::{
    types::{Json, State},
    HttpRequest, HttpResponse,
};

use entities::UserExtension;
use error::{AppResult, CommonError};
use intelli_core::services::UserServiceOperations;
use structs::{UserProfileData, UserUpdateData};

use crate::states::AppState;

#[inline]
pub(crate) async fn get(req: HttpRequest, state: State<AppState>) -> AppResult<HttpResponse> {
    let user = req.user()?;
    let championships = state.user_repo.championships(user.id).await?;

    Ok(HttpResponse::Ok().json(&UserProfileData {
        user,
        championships,
    }))
}

#[inline]
pub(crate) async fn update(
    req: HttpRequest,
    state: State<AppState>,
    user_update: Json<UserUpdateData>,
) -> AppResult<HttpResponse> {
    if user_update.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    };

    let user = req.user()?;
    state.user_svc.update(user, &user_update).await?;
    Ok(HttpResponse::Ok().finish())
}

#[inline]
pub async fn get_championships(
    req: HttpRequest,
    state: State<AppState>,
) -> AppResult<HttpResponse> {
    let user_id = req.user_id()?;
    let championships = state.user_repo.championships(user_id).await?;

    Ok(HttpResponse::Ok().json(&championships))
}

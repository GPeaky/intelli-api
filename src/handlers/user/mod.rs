use garde::Validate;
use ntex::web::{
    types::{Form, State},
    HttpRequest, HttpResponse,
};

use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError},
    states::AppState,
    structs::{UserProfileData, UserUpdateData},
};

pub(crate) mod admin;

#[inline(always)]
pub(crate) async fn get(req: HttpRequest, state: State<AppState>) -> AppResult<HttpResponse> {
    let user = req.user()?;
    let championships = state.championship_repo.find_all(user.id).await?;

    Ok(HttpResponse::Ok().json(&UserProfileData {
        user,
        championships,
    }))
}

#[inline(always)]
pub(crate) async fn update(
    req: HttpRequest,
    state: State<AppState>,
    form: Form<UserUpdateData>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    };

    let user = req.user()?;
    state.user_svc.update(user, &form).await?;
    Ok(HttpResponse::Ok().finish())
}

#[inline(always)]
pub async fn get_championships(
    req: HttpRequest,
    state: State<AppState>,
) -> AppResult<HttpResponse> {
    let user_id = req.user_id()?;
    let championships = state.championship_repo.find_all(user_id).await?;

    Ok(HttpResponse::Ok().json(&championships))
}

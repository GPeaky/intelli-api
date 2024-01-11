use garde::Validate;
use ntex::web::{types, HttpRequest, HttpResponse, Responder};

pub(crate) use admin::*;

use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError},
    services::UserServiceTrait,
    states::AppState,
    structs::{UpdateUser, UserData},
};

mod admin;

#[inline(always)]
pub(crate) async fn user_data(
    req: HttpRequest,
    state: types::State<AppState>,
) -> AppResult<impl Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(HttpResponse::Ok().json(&UserData {
        user,
        championships,
    }))
}

#[inline(always)]
pub(crate) async fn update_user(
    req: HttpRequest,
    state: types::State<AppState>,
    form: types::Form<UpdateUser>,
) -> AppResult<impl Responder> {
    if form.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    };

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    state.user_service.update(&user, &form).await?;
    Ok(HttpResponse::Ok())
}

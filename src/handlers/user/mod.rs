use crate::{
    dtos::{UpdateUser, UserData},
    entity::UserExtension,
    error::{AppResult, CommonError},
    services::UserServiceTrait,
    states::AppState,
};
pub(crate) use admin::*;
use garde::Validate;
use ntex::web;

mod admin;

#[inline(always)]
pub(crate) async fn user_data(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
) -> AppResult<impl web::Responder> {
    let user = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .clone();

    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(web::HttpResponse::Ok().json(&UserData {
        user,
        championships,
    }))
}

#[inline(always)]
pub(crate) async fn update_user(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    form: web::types::Form<UpdateUser>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        Err(CommonError::FormValidationFailed)?
    };

    let user = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .clone();

    state.user_service.update(&user, &form).await?;

    Ok(web::HttpResponse::Ok())
}

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
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

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
        Err(CommonError::ValidationFailed)?
    };

    let user = req
        .extensions()
        .get::<UserExtension>()
        .cloned()
        .ok_or(CommonError::InternalServerError)?;

    state.user_service.update(&user, &form).await?;

    Ok(web::HttpResponse::Ok())
}

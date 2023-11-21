use crate::{
    dtos::UserData,
    entity::UserExtension,
    error::{AppResult, CommonError},
    states::AppState,
};
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

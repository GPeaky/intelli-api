use crate::{
    dtos::VerifyEmailParams, error::AppResult, services::UserServiceTrait, states::AppState,
};
use ntex::web;

#[inline(always)]
pub async fn verify_email(
    state: web::types::State<AppState>,
    query: web::types::Query<VerifyEmailParams>,
) -> AppResult<impl web::Responder> {
    state.user_service.activate_with_token(&query.token).await?;

    Ok(web::HttpResponse::Created())
}

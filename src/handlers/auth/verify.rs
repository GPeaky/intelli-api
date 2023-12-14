use crate::{
    dtos::{EmailUser, EmailVerified, VerifyEmailParams},
    error::AppResult,
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
};
use ntex::web;

#[inline(always)]
pub async fn verify_email(
    state: web::types::State<AppState>,
    query: web::types::Query<VerifyEmailParams>,
) -> AppResult<impl web::Responder> {
    let user_id = state.user_service.activate_with_token(&query.token).await?;
    let user = state.user_repository.find(&user_id).await?.unwrap();

    let template = EmailVerified {};
    let email_user = EmailUser {
        username: &user.username,
        email: &user.email,
    };

    state
        .email_service
        .send_mail(email_user, "Email Verified", template)
        .await?;

    Ok(web::HttpResponse::Created())
}

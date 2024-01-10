use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    error::AppResult,
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::AppState,
    structs::{EmailUser, EmailVerified, VerifyEmailParams},
};

#[inline(always)]
pub async fn verify_email(
    state: State<AppState>,
    query: Query<VerifyEmailParams>,
) -> AppResult<Response> {
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

    Ok(StatusCode::CREATED.into_response())
}

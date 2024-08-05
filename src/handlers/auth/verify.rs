use ntex::web::{
    types::{Query, State},
    HttpResponse,
};

use crate::{
    error::{AppResult, UserError},
    states::AppState,
    structs::{EmailVerified, VerifyEmailParams},
};

#[inline(always)]
pub async fn verify_email(
    state: State<AppState>,
    Query(query): Query<VerifyEmailParams>,
) -> AppResult<HttpResponse> {
    let user_id = state.user_svc.activate_with_token(query.token).await?;
    let user = state
        .user_repo
        .find(user_id)
        .await?
        .ok_or(UserError::NotFound)?;

    let template = EmailVerified {};

    state
        .email_svc
        .send_mail(user, "Email Verified", template)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

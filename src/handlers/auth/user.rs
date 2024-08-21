use chrono::{Duration, Utc};
use garde::Validate;
use ntex::web::{
    types::{Form, Query, State},
    HttpRequest, HttpResponse,
};

use crate::{
    entity::{Provider, UserExtension},
    error::{AppResult, CommonError, UserError},
    states::AppState,
    structs::{
        AuthTokens, ClientFingerprint, EmailVerificationTemplate, LoginCredentials, NewAccessToken,
        PasswordChangeConfirmationTemplate, PasswordResetRequest, PasswordResetTemplate,
        PasswordUpdateData, RefreshTokenRequest, TokenPurpose, TokenVerification,
        UserRegistrationData,
    },
};

// TODO: Add rate limiting to the register endpoint
#[inline(always)]
pub(crate) async fn register(
    state: State<AppState>,
    Form(form): Form<UserRegistrationData>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state.user_svc.create(&form).await?;

    let token = state
        .token_svc
        .generate_token(user_id, TokenPurpose::EmailVerification)?;

    state.token_svc.save_email_token(token.clone());

    // Should be safe to unwrap the option cause we just created the user above
    let user = state.user_repo.find(user_id).await?.unwrap();

    let template = EmailVerificationTemplate {
        verification_link: &format!(
            "https://intellitelemetry.live/auth/verify-email?token={}",
            token
        ),
    };

    state
        .email_svc
        .send_mail(user, "Verify Email", template)
        .await?;

    Ok(HttpResponse::Created().finish())
}

#[inline(always)]
pub(crate) async fn login(
    state: State<AppState>,
    Query(query): Query<ClientFingerprint>,
    Form(form): Form<LoginCredentials>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repo.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if user.provider != Provider::Local {
        return Err(UserError::GoogleLogin)?;
    }

    if !state
        .user_repo
        .validate_password(form.password, user.password.clone().unwrap())
        .await?
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let access_token = state
        .token_svc
        .generate_token(user.id, TokenPurpose::Authentication)?;

    let refresh_token = state
        .token_svc
        .generate_refresh_token(user.id, query.fingerprint)?;

    let auth_response = AuthTokens {
        access_token,
        refresh_token,
    };

    Ok(HttpResponse::Ok().json(&auth_response))
}

#[inline(always)]
pub(crate) async fn refresh_token(
    state: State<AppState>,
    Query(query): Query<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let access_token = state
        .token_svc
        .refresh_access_token(&query.refresh_token, query.fingerprint)?;

    let refresh_response = NewAccessToken { access_token };

    Ok(HttpResponse::Ok().json(&refresh_response))
}

#[inline(always)]
pub(crate) async fn logout(
    req: HttpRequest,
    state: State<AppState>,
    Query(query): Query<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let user_id = req.user_id()?;

    state
        .token_svc
        .remove_refresh_token(user_id, query.fingerprint);

    Ok(HttpResponse::Ok().finish())
}

#[inline(always)]
pub(crate) async fn forgot_password(
    state: State<AppState>,
    form: Form<PasswordResetRequest>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repo.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if Utc::now().signed_duration_since(user.updated_at) > Duration::try_hours(1).unwrap() {
        return Err(UserError::UpdateLimitExceeded)?;
    }

    let token = state
        .token_svc
        .generate_token(user.id, TokenPurpose::PasswordReset)?;

    let template = PasswordResetTemplate {
        reset_password_link: &format!(
            "https://intellitelemetry.live/auth/reset-password?token={}",
            token
        ),
    };

    state.token_svc.save_reset_password_token(token);

    state
        .email_svc
        .send_mail(user, "Reset Password", template)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

// TODO: Add rate limiting to the reset password endpoint
#[inline(always)]
pub async fn reset_password(
    state: State<AppState>,
    Query(query): Query<TokenVerification>,
    Form(form): Form<PasswordUpdateData>,
) -> AppResult<HttpResponse> {
    if form.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state
        .user_svc
        .reset_password_with_token(query.token, form.password)
        .await?;

    let Some(user) = state.user_repo.find(user_id).await? else {
        Err(UserError::NotFound)?
    };

    let template = PasswordChangeConfirmationTemplate {};

    state
        .email_svc
        .send_mail(user, "Password Changed", template)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

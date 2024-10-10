use chrono::{Duration, Utc};
use garde::Validate;
use ntex::web::{
    types::{Json, Query, State},
    HttpResponse,
};

use entities::Provider;
use error::{AppResult, CommonError, UserError};
use intelli_core::services::UserServiceOperations;
use structs::{
    AuthTokens, ClientFingerprint, EmailVerificationTemplate, LoginCredentials, NewAccessToken,
    PasswordChangeConfirmationTemplate, PasswordResetRequest, PasswordResetTemplate,
    PasswordUpdateData, RefreshTokenRequest, TokenVerification, UserRegistrationData,
};
use token::{Token, TokenIntent};

use crate::states::AppState;

// TODO: Add rate limiting to the register endpoint
#[inline]
pub(crate) async fn register(
    state: State<AppState>,
    Json(user_registration): Json<UserRegistrationData>,
) -> AppResult<HttpResponse> {
    if user_registration.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state.user_svc.create(user_registration).await?;

    let token = state.token_mgr.create(user_id, TokenIntent::EmailVerify);

    // Should be safe to unwrap the option because we just created the user above
    let user = state.user_repo.find(user_id).await?.unwrap();

    let template = EmailVerificationTemplate {
        verification_link: format!(
            "https://intellitelemetry.live/auth/verify-email?token={}",
            token.as_base64()
        ),
    };

    state
        .email_svc
        .send_mail(user, "Verify Email", template)
        .await?;

    Ok(HttpResponse::Created().finish())
}

#[inline]
pub(crate) async fn login(
    state: State<AppState>,
    Query(_query): Query<ClientFingerprint>,
    Json(login_credentials): Json<LoginCredentials>,
) -> AppResult<HttpResponse> {
    if login_credentials.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state
        .user_repo
        .find_by_email(&login_credentials.email)
        .await?
    else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if user.provider != Provider::Local {
        return Err(UserError::DiscordAuth)?;
    }

    if !state
        .user_repo
        .validate_password(login_credentials.password, user.password.clone().unwrap())
        .await?
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let access_token = state.token_mgr.create(user.id, TokenIntent::Auth);
    let refresh_token = state.token_mgr.create(user.id, TokenIntent::RefreshAuth);

    let auth_response = AuthTokens {
        access_token: access_token.as_base64(),
        refresh_token: refresh_token.as_base64(),
    };

    Ok(HttpResponse::Ok().json(&auth_response))
}

#[inline]
pub(crate) async fn refresh_token(
    state: State<AppState>,
    Query(query): Query<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let token = Token::from_base64(&query.refresh_token)?;

    let user_id = state.token_mgr.validate(&token, TokenIntent::RefreshAuth)?;

    let access_token = state.token_mgr.create(user_id, TokenIntent::Auth);

    let refresh_response = NewAccessToken {
        access_token: access_token.as_base64(),
    };

    Ok(HttpResponse::Ok().json(&refresh_response))
}

#[inline]
pub(crate) async fn logout(
    state: State<AppState>,
    Query(query): Query<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let token = Token::from_base64(&query.refresh_token)?;
    state.token_mgr.remove(&token, TokenIntent::RefreshAuth);

    Ok(HttpResponse::Ok().finish())
}

#[inline]
pub(crate) async fn forgot_password(
    state: State<AppState>,
    password_reset: Json<PasswordResetRequest>,
) -> AppResult<HttpResponse> {
    if password_reset.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repo.find_by_email(&password_reset.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if let Some(last_update) = user.updated_at {
        if Utc::now().signed_duration_since(last_update) > Duration::hours(1) {
            return Err(UserError::UpdateLimitExceeded)?;
        }
    }

    let token = state.token_mgr.create(user.id, TokenIntent::PasswordReset);

    let template = PasswordResetTemplate {
        reset_password_link: format!(
            "https://intellitelemetry.live/auth/reset-password?token={}",
            token.as_base64()
        ),
    };

    state
        .email_svc
        .send_mail(user, "Reset Password", template)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

// TODO: Add rate limiting to the reset password endpoint
#[inline]
pub async fn reset_password(
    state: State<AppState>,
    Query(query): Query<TokenVerification>,
    Json(password_update): Json<PasswordUpdateData>,
) -> AppResult<HttpResponse> {
    if password_update.validate().is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state
        .user_svc
        .reset_password(query.token, password_update.password)
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

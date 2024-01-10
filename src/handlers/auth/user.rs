use axum::{
    extract::{Extension, Form, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use garde::Validate;

use crate::{
    entity::{Provider, UserExtension},
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AppState,
    structs::{
        AuthResponse, EmailUser, FingerprintQuery, ForgotPasswordDto, LoginUserDto,
        PasswordChanged, RefreshResponse, RefreshTokenQuery, RegisterUserDto, ResetPassword,
        ResetPasswordDto, ResetPasswordQuery, TokenType, VerifyEmail,
    },
};

#[inline(always)]
pub(crate) async fn register(
    state: State<AppState>,
    form: Form<RegisterUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state.user_service.create(&form).await?;

    let token = state
        .token_service
        .generate_token(user_id, TokenType::Email)
        .await?;

    let template = VerifyEmail {
        verification_link: &format!(
            "https://intellitelemetry.live/auth/verify-email?token={}",
            token
        ),
    };

    let save_email_future = state.token_service.save_email_token(&token);

    let send_email_future =
        state
            .email_service
            .send_mail((&*form).into(), "Verify Email", template);

    tokio::try_join!(save_email_future, send_email_future)?;
    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub(crate) async fn login(
    state: State<AppState>,
    query: Query<FingerprintQuery>,
    form: Form<LoginUserDto>,
) -> AppResult<Json<AuthResponse>> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if user.provider != Provider::Local {
        return Err(UserError::GoogleLogin)?;
    }

    if !state
        .user_repository
        .validate_password(&form.password, &user.password.unwrap())?
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let access_token_future = state
        .token_service
        .generate_token(user.id, TokenType::Bearer);

    let refresh_token_future = state
        .token_service
        .generate_refresh_token(&user.id, &query.fingerprint);

    let (access_token, refresh_token) =
        tokio::try_join!(access_token_future, refresh_token_future)?;

    let auth_response = AuthResponse {
        access_token,
        refresh_token,
    };

    Ok(Json(auth_response))
}

#[inline(always)]
pub(crate) async fn refresh_token(
    state: State<AppState>,
    query: Query<RefreshTokenQuery>,
) -> AppResult<Json<RefreshResponse>> {
    let access_token = state
        .token_service
        .refresh_access_token(&query.refresh_token, &query.fingerprint)
        .await?;

    let refresh_response = RefreshResponse { access_token };
    Ok(Json(refresh_response))
}

#[inline(always)]
pub(crate) async fn logout(
    state: State<AppState>,
    user: Extension<UserExtension>,
    query: Query<FingerprintQuery>,
) -> AppResult<Response> {
    state
        .token_service
        .remove_refresh_token(&user.id, &query.fingerprint)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub(crate) async fn forgot_password(
    state: State<AppState>,
    form: Form<ForgotPasswordDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if Utc::now().signed_duration_since(user.updated_at) > Duration::hours(1) {
        return Err(UserError::UpdateLimitExceeded)?;
    }

    let token = state
        .token_service
        .generate_token(user.id, TokenType::ResetPassword)
        .await?;

    let template = ResetPassword {
        reset_password_link: &format!(
            "https://intellitelemetry.live/auth/reset-password?token={}",
            token
        ),
    };

    let save_reset_password = state.token_service.save_reset_password_token(&token);

    let send_mail = state.email_service.send_mail(
        EmailUser {
            username: &user.username,
            email: &user.email,
        },
        "Reset Password",
        template,
    );

    tokio::try_join!(save_reset_password, send_mail)?;
    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn reset_password(
    query: Query<ResetPasswordQuery>,
    state: State<AppState>,
    form: Form<ResetPasswordDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state
        .user_service
        .reset_password_with_token(&query.token, &form.password)
        .await?;

    let Some(user) = state.user_repository.find(&user_id).await? else {
        Err(UserError::NotFound)?
    };

    let template = PasswordChanged {};

    state
        .email_service
        .send_mail(
            EmailUser {
                username: &user.username,
                email: &user.email,
            },
            "Password Changed",
            template,
        )
        .await?;

    Ok(StatusCode::OK.into_response())
}

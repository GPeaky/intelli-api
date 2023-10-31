use crate::{
    dtos::{
        AuthResponse, EmailUser, FingerprintQuery, ForgotPasswordDto, LoginUserDto,
        PasswordChanged, RefreshResponse, RefreshTokenQuery, RegisterUserDto, ResetPassword,
        ResetPasswordDto, ResetPasswordQuery, TokenType, VerifyEmail,
    },
    entity::UserExtension,
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AuthState,
};
use axum::{
    extract::{Form, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use garde::Validate;

#[inline(always)]
pub(crate) async fn register(
    State(state): State<AuthState>,
    Form(form): Form<RegisterUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let user_id = state.user_service.new_user(&form).await?;

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

    let save_email = state.token_service.save_email_token(&token);

    let send_email = state
        .email_service
        .send_mail((&form).into(), "Verify Email", template);

    tokio::try_join!(save_email, send_email)?;

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub(crate) async fn login(
    State(state): State<AuthState>,
    Query(query): Query<FingerprintQuery>,
    Form(form): Form<LoginUserDto>,
) -> AppResult<Json<AuthResponse>> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if !state
        .user_repository
        .validate_password(&form.password, &user.password.unwrap())
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

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
    }))
}

#[inline(always)]
pub(crate) async fn refresh_token(
    State(state): State<AuthState>,
    Query(query): Query<RefreshTokenQuery>,
) -> AppResult<Json<RefreshResponse>> {
    let new_token = state
        .token_service
        .refresh_access_token(&query.refresh_token, &query.fingerprint)
        .await?;

    Ok(Json(RefreshResponse {
        access_token: new_token,
    }))
}

#[inline(always)]
pub(crate) async fn logout(
    State(state): State<AuthState>,
    Extension(user): Extension<UserExtension>,
    Query(query): Query<FingerprintQuery>,
) -> AppResult<Response> {
    state
        .token_service
        .remove_refresh_token(&user.id, &query.fingerprint)
        .await?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub(crate) async fn forgot_password(
    State(state): State<AuthState>,
    Form(form): Form<ForgotPasswordDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

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
    Query(query): Query<ResetPasswordQuery>,
    State(state): State<AuthState>,
    Form(form): Form<ResetPasswordDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
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

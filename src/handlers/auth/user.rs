use crate::{
    dtos::{
        AuthResponse, EmailUser, FingerprintQuery, ForgotPasswordDto, LoginUserDto,
        RefreshResponse, RefreshTokenQuery, RegisterUserDto, ResetPassword, ResetPasswordDto,
        ResetPasswordQuery, TokenType, VerifyEmail,
    },
    entity::UserExtension,
    error::{AppResult, CommonError, TokenError, UserError},
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
        verification_link: &format!("https://intellitelemetry.live/auth/verify-email/{}", token),
    };

    state
        .email_service
        .send_mail(&(&form).into(), "Verify Email", template)
        .await
        .map_err(|_| UserError::MailError)?;

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

// TODO: Save the token in the database
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
            "https://intellitelemetry.live/auth/reset-password/{}",
            token
        ),
    };

    state
        .email_service
        .send_mail(
            &EmailUser {
                username: &user.username,
                email: &user.email,
            },
            "Reset Password",
            template,
        )
        .await
        .map_err(|_| UserError::MailError)?;

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

    let token_data = state.token_service.validate(&query.token)?;

    if token_data.claims.token_type.ne(&TokenType::ResetPassword) {
        Err(TokenError::InvalidTokenType)?
    }

    // TODO: Check if token is on the db and search user by id and change password
    Ok(StatusCode::OK.into_response())
}

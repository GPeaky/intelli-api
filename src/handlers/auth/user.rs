use crate::{
    dtos::{
        AuthResponse, EmailUser, ForgotPasswordDto, LoginUserDto, RefreshResponse, RegisterUserDto,
        ResetPasswordDto, ResetPasswordQuery, TokenType,
    },
    entity::User,
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
use hyper::HeaderMap;

#[inline(always)]
pub(crate) async fn register(
    State(state): State<AuthState>,
    Form(form): Form<RegisterUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let user_id = state.user_service.new_user(&form).await?.to_string();

    let token = state
        .token_service
        .generate_token(&user_id, TokenType::Email)
        .await?;

    state
        .email_service
        .send_mail(
            &(&form).into(),
            "Verify Email",
            format!(
                "Click on the link to verify your email: http://localhost:3000/verify-email/{}",
                token
            ),
        )
        .await
        .map_err(|_| UserError::MailError)?;

    Ok(StatusCode::CREATED.into_response())
}

#[inline(always)]
pub(crate) async fn login(
    headers: HeaderMap,
    State(state): State<AuthState>,
    Form(form): Form<LoginUserDto>,
) -> AppResult<Json<AuthResponse>> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let fingerprint = headers
        .get("Fingerprint")
        .ok_or(UserError::InvalidFingerprint)?
        .to_str()
        .map_err(|_| UserError::InvalidFingerprint)?;

    let user = state
        .user_repository
        .find_by_email(&form.email)
        .await
        .map_err(|_| UserError::NotFound)?;

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if !state
        .user_repository
        .validate_password(&form.password, &user.password)
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let user_id = user.id.to_string();

    let access_token_future = state
        .token_service
        .generate_token(&user_id, TokenType::Bearer);

    let refresh_token_future = state
        .token_service
        .generate_refresh_token(&user_id, fingerprint);

    let (access_token, refresh_token) =
        tokio::try_join!(access_token_future, refresh_token_future)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
    }))
}

#[inline(always)]
pub(crate) async fn refresh_token(
    headers: HeaderMap,
    State(state): State<AuthState>,
) -> AppResult<Json<RefreshResponse>> {
    let fingerprint = headers
        .get("Fingerprint")
        .ok_or(UserError::InvalidFingerprint)?
        .to_str()
        .map_err(|_| UserError::InvalidFingerprint)?;

    // TODO: Change this to a more generic way. Like query params
    let refresh_token = headers
        .get("RefreshToken")
        .ok_or(UserError::InvalidRefreshToken)?
        .to_str()
        .map_err(|_| UserError::InvalidRefreshToken)?;

    let new_token = state
        .token_service
        .refresh_access_token(refresh_token, fingerprint)
        .await?;

    Ok(Json(RefreshResponse {
        access_token: new_token,
    }))
}

#[inline(always)]
pub(crate) async fn logout(
    headers: HeaderMap,
    State(state): State<AuthState>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    let fingerprint = headers
        .get("Fingerprint")
        .ok_or(UserError::InvalidFingerprint)?
        .to_str()
        .map_err(|_| UserError::InvalidFingerprint)?;

    state
        .token_service
        .remove_refresh_token(user.id.to_string(), fingerprint)
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

    let user = state.user_repository.find_by_email(&form.email).await?;

    let token = state
        .token_service
        .generate_token(&user.id.to_string(), TokenType::ResetPassword)
        .await?;

    state
        .email_service
        .send_mail(
            &EmailUser {
                username: &user.username,
                email: &user.email,
            },
            "Reset Password",
            format!(
                "Click on the link to reset your password: http://localhost:3000/reset-password{}",
                token
            ),
        )
        .await
        .map_err(|_| UserError::MailError)?;

    Ok(StatusCode::OK.into_response())
}

#[inline(always)]
pub async fn reset_password(
    Query(ResetPasswordQuery { token }): Query<ResetPasswordQuery>,
    State(state): State<AuthState>,
    Form(form): Form<ResetPasswordDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let token_data = state.token_service.validate(&token)?;

    if token_data.claims.token_type.ne(&TokenType::ResetPassword) {
        Err(TokenError::InvalidTokenType)?
    }

    // TODO: Check if toke is on the db and search user by id and change password
    Ok(StatusCode::OK.into_response())
}

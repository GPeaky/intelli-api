use crate::{
    dtos::{
        AuthResponse, EmailUser, ForgotPasswordDto, LoginUserDto, RefreshResponse, RegisterUserDto,
        ResetPasswordDto, ResetPasswordQuery, ResetPasswordTemplate, Templates, TokenType,
        VerifyEmailTemplate,
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

    state.user_service.new_user(&form).await?;

    let token = state
        .token_service
        .generate_token(&form.email, TokenType::Email)?;

    state
        .email_service
        .send_mail(
            // TODO: Remove this unnecessary clone
            &form.clone().into(),
            Templates::VerifyEmail(VerifyEmailTemplate {
                username: &form.username,
                token: &token,
            }),
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

    let access_token = state
        .token_service
        .generate_token(&user.id, TokenType::Bearer)?;

    let refresh_token = state
        .token_service
        .generate_refresh_token(user.id, fingerprint)
        .await?;

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
        .remove_refresh_token(user.id, fingerprint)
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
        .generate_token(&user.id, TokenType::ResetPassword)?;

    state
        .email_service
        .send_mail(
            &EmailUser {
                // TODO: Check this unnecessary clone
                username: user.username.clone(),
                email: user.email.clone(),
            },
            Templates::ResetPassword(ResetPasswordTemplate {
                name: &user.username,
                token: &token,
            }),
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

    if token_data.claims.token_type != TokenType::ResetPassword {
        Err(TokenError::InvalidTokenType)?
    }

    // TODO: Check if toke is on the db and search user by id and change password

    Ok(StatusCode::OK.into_response())
}

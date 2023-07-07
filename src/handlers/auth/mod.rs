use crate::{
    dtos::{AuthResponse, LoginUserDto, RegisterUserDto, TokenType},
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AuthState,
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;

pub(crate) async fn register(
    State(state): State<AuthState>,
    Form(form): Form<RegisterUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    state.user_service.new_user(form).await?;

    Ok(StatusCode::OK.into_response())
}

pub(crate) async fn login(
    State(state): State<AuthState>,
    Form(form): Form<LoginUserDto>,
) -> AppResult<Json<AuthResponse>> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    let user = state
        .user_repository
        .find(&form.email)
        .await
        .map_err(|_| UserError::NotFound)?;

    if !state
        .user_repository
        .validate_password(&form.password, &user.password)
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let access_token = state
        .token_service
        .generate_token(user.id.clone(), TokenType::Bearer)?;

    let refresh_token = state
        .token_service
        .generate_refresh_token(user.id, "refresh")
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
    }))
}

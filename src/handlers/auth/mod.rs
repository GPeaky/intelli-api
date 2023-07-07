use crate::{
    dtos::{LoginUserDto, RegisterUserDto},
    error::{AppResult, CommonError},
    services::UserServiceTrait,
    states::AuthState,
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use garde::Validate;

pub(crate) async fn register(
    State(state): State<AuthState>,
    Form(form): Form<RegisterUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    state.user_service.register(form).await?;

    Ok(StatusCode::OK.into_response())
}

pub(crate) async fn login(
    State(state): State<AuthState>,
    Form(form): Form<LoginUserDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    state.user_service.login(form).await?;

    Ok(StatusCode::OK.into_response())
}

use crate::{
    dtos::{LoginUserDto, RegisterUserDto},
    error::AppResult,
    services::UserServiceTrait,
    states::AuthState,
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub(crate) async fn register(
    State(state): State<AuthState>,
    Form(form): Form<RegisterUserDto>,
) -> AppResult<Response> {
    state.user_service.register(form).await?;

    Ok(StatusCode::OK.into_response())
}

pub(crate) async fn login(
    State(_state): State<AuthState>,
    Form(_form): Form<LoginUserDto>,
) -> AppResult<Response> {
    // state.user_service.login(form).await?;

    Ok(StatusCode::OK.into_response())
}

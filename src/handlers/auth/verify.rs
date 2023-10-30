use crate::{
    dtos::{TokenType, VerifyEmailParams},
    error::{AppResult, TokenError},
    services::{TokenServiceTrait, UserServiceTrait},
    states::AuthState,
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use tracing::info;

#[inline(always)]
pub async fn verify_email(
    State(state): State<AuthState>,
    Query(query): Query<VerifyEmailParams>,
) -> AppResult<Response> {
    let token_data = state.token_service.validate(&query.token)?;
    info!("{:?}", token_data.claims);
    if token_data.claims.token_type.ne(&TokenType::Email) {
        Err(TokenError::InvalidToken)?
    }
    info!("User id: {}", token_data.claims.sub);

    state
        .user_service
        .activate_user(&token_data.claims.sub)
        .await
        .unwrap();

    Ok(StatusCode::ACCEPTED.into_response())
}

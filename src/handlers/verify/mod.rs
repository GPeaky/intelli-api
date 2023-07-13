use crate::{
    dtos::TokenType,
    error::{AppResult, TokenError},
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AuthState,
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyEmailParams {
    token: String,
}

#[inline]
pub async fn verify_email(
    State(state): State<AuthState>,
    Query(query): Query<VerifyEmailParams>,
) -> AppResult<Response> {
    let token_data = state.token_service.validate(&query.token)?;

    if token_data.claims.token_type != TokenType::Email {
        return Err(TokenError::InvalidToken)?;
    }

    // FIX: Check if we could skip this part
    let user = state
        .user_repository
        .find_by_email(&token_data.claims.sub)
        .await?;

    state
        .user_service
        .verify_email(&user.id, &token_data.claims.sub)
        .await
        .unwrap();

    Ok(StatusCode::ACCEPTED.into_response())
}

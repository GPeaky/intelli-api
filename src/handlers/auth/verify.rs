use crate::{
    dtos::VerifyEmailParams, error::AppResult, services::UserServiceTrait, states::AuthState,
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;

#[inline(always)]
pub async fn verify_email(
    State(state): State<AuthState>,
    Query(query): Query<VerifyEmailParams>,
) -> AppResult<Response> {
    state
        .user_service
        .activate_with_token(&query.token)
        .await
        .unwrap();

    Ok(StatusCode::ACCEPTED.into_response())
}

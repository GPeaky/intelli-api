use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    error::{AppResult, TokenError},
    repositories::UserRepositoryTrait,
    services::TokenServiceTrait,
    states::AppState,
};

const BEARER_PREFIX: &str = "Bearer ";

pub async fn auth_handler(
    state: State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let token = req
        .headers()
        .get("Authorization")
        .ok_or(TokenError::MissingToken)?;

    let token = token.to_str().map_err(|_| TokenError::InvalidToken)?;

    if !token.starts_with(BEARER_PREFIX) {
        Err(TokenError::InvalidToken)?
    }

    let token = token.trim_start_matches(BEARER_PREFIX);
    let token = state.token_service.validate(token)?;

    let Some(user) = state.user_repository.find(&token.claims.sub).await? else {
        Err(TokenError::InvalidToken)?
    };

    req.extensions_mut().insert(Arc::from(user));
    Ok(next.run(req).await)
}

use crate::{
    error::{AppResult, TokenError, UserError},
    repositories::UserRepositoryTrait,
    services::TokenServiceTrait,
    states::UserState,
};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Request},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_handler<T>(
    State(state): State<UserState>,
    mut req: Request<T>,
    next: Next<T>,
) -> AppResult<Response> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(TokenError::MissingToken)?;

    let mut header = header.to_str().map_err(|_| TokenError::InvalidToken)?;
    if !header.starts_with("Bearer ") {
        return Err(TokenError::InvalidToken)?;
    }

    header = &header[7..];

    let token = state.token_service.validate(header)?;

    let Some(user) = state.user_repository.find(&token.claims.sub).await? else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    req.extensions_mut().insert(Arc::new(user));
    Ok(next.run(req).await)
}

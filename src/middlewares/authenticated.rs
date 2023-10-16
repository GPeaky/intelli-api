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
    let header_value = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(TokenError::MissingToken)?;

    let header_str = header_value
        .to_str()
        .map_err(|_| TokenError::InvalidToken)?;

    // Verificar que el encabezado comienza con "Bearer "
    if !header_str.starts_with("Bearer ") {
        return Err(TokenError::InvalidToken)?;
    }

    // Extraer el token real
    let extracted_token = &header_str[7..];

    // Validar el token
    let token = state.token_service.validate(extracted_token)?;

    let Some(user) = state
        .user_repository
        .find(&token.claims.sub.parse::<u32>().unwrap())
        .await?
    else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    req.extensions_mut().insert(Arc::new(user));
    Ok(next.run(req).await)
}

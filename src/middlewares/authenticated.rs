use crate::{
    error::{AppResult, TokenError, UserError},
    repositories::UserRepositoryTrait,
    services::TokenServiceTrait,
    states::UserState,
};
use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization, Header},
    http::{header::AUTHORIZATION, Request},
    middleware::Next,
    response::Response,
};

pub async fn auth_handler<T>(
    State(state): State<UserState>,
    mut req: Request<T>,
    next: Next<T>,
) -> AppResult<Response> {
    let mut headers = req
        .headers_mut()
        .iter()
        .filter_map(|(header_name, header_value)| {
            if header_name == AUTHORIZATION {
                return Some(header_value);
            }

            None
        });

    let header: Authorization<Bearer> =
        Authorization::decode(&mut headers).map_err(|_| TokenError::InvalidToken)?;

    let Ok(token) = state.token_service.validate(header.token()) else {
        Err(TokenError::InvalidToken)?
    };

    let user = state
        .user_repository
        .find(&token.claims.sub.parse::<i32>().unwrap())
        .await
        .map_err(|_| UserError::NotFound)?;

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

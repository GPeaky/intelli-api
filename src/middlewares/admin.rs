use axum::{body::Body, http::Request, middleware::Next, response::Response};
use tracing::info;

use crate::{
    entity::{Role, UserExtension},
    error::{AppResult, UserError},
};

pub async fn admin_handler(request: Request<Body>, next: Next) -> AppResult<Response> {
    let user = request
        .extensions()
        .get::<UserExtension>()
        .ok_or(UserError::Unauthorized)?;

    if user.role != Role::Admin {
        info!("User {} is not admin", user.id);
        Err(UserError::Unauthorized)?
    }

    Ok(next.run(request).await)
}

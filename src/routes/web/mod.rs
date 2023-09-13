use crate::handlers::web::{html_index, login, register};
use axum::{routing::get, Router};
use tower_http::{compression::CompressionLayer, services::ServeDir};

#[inline(always)]
pub(crate) fn web_router() -> Router {
    let auth_router = Router::new()
        .route("/login", get(login))
        .route("/register", get(register));

    Router::new()
        .route("/", get(html_index))
        .nest("/auth", auth_router)
        .nest_service("/public", ServeDir::new("public"))
        .layer(
            CompressionLayer::new()
                .br(true)
                .quality(tower_http::CompressionLevel::Fastest),
        ) // TODO: Check if this is really needed
}

use axum::{routing::get, Router};
use tower_http::services::ServeDir;

use crate::handlers::web::html_index;

#[inline(always)]
pub(crate) fn web_router() -> Router {
    Router::new()
        .route("/", get(html_index))
        .nest_service("/public", ServeDir::new("public"))
}

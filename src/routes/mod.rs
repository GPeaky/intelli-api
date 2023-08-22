use crate::config::Database;
use axum::{routing::IntoMakeService, Router};
use std::sync::Arc;

mod api;
mod web;

pub(crate) async fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    Router::new()
        .nest("/", web::web_router())
        .nest("/api", api::api_router(database).await)
        .into_make_service()
}

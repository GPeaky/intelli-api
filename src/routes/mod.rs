use crate::config::Database;
use axum::{error_handling::HandleErrorLayer, routing::IntoMakeService, Router};
use hyper::StatusCode;
use std::{sync::Arc, time::Duration};
use tower::{load_shed::LoadShedLayer, ServiceBuilder};

mod api;

// Handles Service Errors
pub async fn handle_error(e: Box<dyn std::error::Error + Send + Sync>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", e),
    )
}

pub(crate) async fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    Router::new()
        .nest("/", api::api_router(database).await)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(LoadShedLayer::new())
                .timeout(Duration::from_secs(10)),
        )
        .into_make_service()
}

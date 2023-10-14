use crate::config::Database;
use axum::{error_handling::HandleErrorLayer, http::HeaderValue, routing::IntoMakeService, Router};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use hyper::{Method, StatusCode};
use std::{sync::Arc, time::Duration};
use tower::{load_shed::LoadShedLayer, ServiceBuilder};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

mod api;

// Handles Service Errors
pub async fn handle_error(e: Box<dyn std::error::Error + Send + Sync>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", e),
    )
}

#[inline(always)]
pub(crate) async fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    let metrics = HttpMetricsLayerBuilder::new()
        .with_service_name("intelli_api".to_owned())
        .with_service_version("0.1.0".to_owned())
        .build();

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(vec![
            HeaderValue::from_static("https://intellitelemetry.live"),
            HeaderValue::from_static("http://localhost:5173"),
        ]))
        .allow_headers(AllowHeaders::any())
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::PUT]);

    Router::new()
        .nest("/", api::api_router(database).await)
        .merge(metrics.routes())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(LoadShedLayer::new())
                .layer(cors_layer)
                .layer(metrics)
                .timeout(Duration::from_secs(4)),
        )
        .into_make_service()
}

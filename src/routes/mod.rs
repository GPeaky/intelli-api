use axum::{
    routing::{get, IntoMakeService},
    Router,
};

use crate::handlers::init;

pub(crate) fn service_routes() -> IntoMakeService<Router> {
    Router::new().route("/", get(init)).into_make_service()
}

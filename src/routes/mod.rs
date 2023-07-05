use axum::{
    routing::{get, IntoMakeService},
    Router,
};

use crate::{config::Database, handlers::init};

pub(crate) fn service_routes(_database: Database) -> IntoMakeService<Router> {
    Router::new().route("/", get(init)).into_make_service()
}

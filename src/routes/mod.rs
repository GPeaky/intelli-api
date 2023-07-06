use axum::{
    routing::{get, post, IntoMakeService},
    Router,
};

use crate::{config::Database, handlers::init};

pub(crate) fn service_routes(_database: Database) -> IntoMakeService<Router> {
    let auth_router = Router::new()
        .route("/register", post(init))
        .route("/login", post(init));

    Router::new().nest("/auth", auth_router).into_make_service()
}

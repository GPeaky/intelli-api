use std::sync::Arc;

use axum::{
    routing::{post, IntoMakeService},
    Router,
};

use crate::{
    config::Database,
    handlers::auth::{login, register},
    states::AuthState,
};

pub(crate) fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    let auth_state = AuthState::new(&database);

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(auth_state);

    Router::new().nest("/auth", auth_router).into_make_service()
}

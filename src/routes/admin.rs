use crate::{
    handlers::{
        championships::active_sockets,
        user::{delete_user, disable_user, enable_user},
    },
    middlewares::{admin_handler, auth_handler},
    states::UserState,
};
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

pub fn admin_router(state: UserState) -> Router {
    let socket_router = Router::new().route("/sockets", get(active_sockets));

    let user_router = Router::new()
        .route("/:id", delete(delete_user))
        .route("/:id/enable", post(enable_user))
        .route("/:id/disable", post(disable_user));

    Router::new()
        .nest("/users", user_router)
        .nest("/sockets", socket_router)
        .route_layer(middleware::from_fn(admin_handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_handler))
        .with_state(state)
}

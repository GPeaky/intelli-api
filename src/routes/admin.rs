use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::{
    handlers::{
        admin::pool_status,
        championships::{active_sockets, delete_championship, user_championships},
        user::{delete_user, disable_user, enable_user},
    },
    middlewares::admin_handler,
    states::AppState,
};

pub(crate) fn admin_routes() -> Router<AppState> {
    let admin_middleware = middleware::from_fn(admin_handler);

    let admin_user_router = Router::new()
        .route("/:id", delete(delete_user))
        .route("/:id/enable", post(enable_user))
        .route("/:id/disable", post(disable_user));

    let admin_championship_router = Router::new()
        .route("/:id", get(user_championships))
        .route("/:id", delete(delete_championship));

    Router::new()
        .route("/pools", get(pool_status))
        .route("/sockets", get(active_sockets))
        .nest("/users", admin_user_router)
        .nest("/championships", admin_championship_router)
        .route_layer(admin_middleware)
}

use crate::{
    handlers::{
        admin::pool_status,
        championships::{
            active_sockets, delete_championship, update_championship, user_championships,
        },
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

#[inline(always)]
pub fn admin_router(state: UserState) -> Router {
    let socket_router = Router::new().route("/sockets", get(active_sockets));

    let user_router = Router::new()
        .route("/:id", delete(delete_user))
        .route("/:id/enable", post(enable_user))
        .route("/:id/disable", post(disable_user));

    let championships_router = Router::new()
        .route("/:id", get(user_championships)) // id = user_id
        .route("/:id", delete(delete_championship))
        .route("/:id", post(update_championship));

    Router::new()
        .nest("/users", user_router)
        .nest("/championships", championships_router)
        .nest("/sockets", socket_router)
        .route("/pools", get(pool_status))
        .route_layer(middleware::from_fn(admin_handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_handler))
        .with_state(state)
}

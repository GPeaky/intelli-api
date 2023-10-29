use self::admin::admin_router;
use super::handle_error;
use crate::handlers::auth::callback;
use crate::handlers::championships::socket_status;
use crate::handlers::heartbeat;
use crate::services::FirewallService;
use crate::states::{AuthStateInner, UserStateInner};
use crate::{
    config::Database,
    handlers::{
        auth::{
            forgot_password, login, logout, refresh_token, register, reset_password, verify_email,
        },
        championships::{
            all_championships, create_championship, get_championship, session_socket, start_socket,
            stop_socket,
        },
        user::user_data,
    },
    middlewares::auth_handler,
};
use axum::{
    error_handling::HandleErrorLayer,
    middleware,
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;

mod admin;

#[inline(always)]
pub(crate) async fn api_router(
    database: Arc<Database>,
    firewall_service: Arc<FirewallService>,
) -> Router {
    let auth_state = Arc::new(AuthStateInner::new(&database));
    let user_state = Arc::new(UserStateInner::new(&database, firewall_service).await);

    let auth_middleware = middleware::from_fn_with_state(user_state.clone(), auth_handler);

    let auth_router = Router::new()
        .route("/google/callback", get(callback))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", get(refresh_token))
        .route("/verify/email", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/logout", get(logout).route_layer(auth_middleware.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .buffer(1024)
                .rate_limit(8, Duration::from_secs(60)),
        )
        .with_state(auth_state);

    let user_router = Router::new()
        .route("/data", get(user_data))
        .with_state(user_state.clone())
        .route_layer(auth_middleware.clone());

    // todo: Delete Championship, Get Championship
    // todo: Add Round to Championship, Delete Round from Championship, Get Round from Championship and reference it to the corresponding session
    let championships_router = Router::new()
        .route("/", post(create_championship))
        .route("/:id", get(get_championship))
        .route("/all", get(all_championships))
        .route("/:id/socket/start", get(start_socket))
        .route("/:id/socket/status", get(socket_status))
        .route("/:id/socket/stop", get(stop_socket))
        .route_layer(auth_middleware)
        .with_state(user_state.clone());

    Router::new()
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .nest("/championships", championships_router)
        .nest("/admin", admin_router(user_state.clone()))
        .route("/heartbeat", get(heartbeat))
        .route(
            "/championships/:id/web_socket", // Removed /session/session:id to make it easier to use
            get(session_socket).with_state(user_state),
        )
}

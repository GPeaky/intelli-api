use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handlers::{
        auth::{
            callback, forgot_password, login, logout, refresh_token, register, reset_password,
            verify_email,
        },
        championships::{
            all_championships, create_championship, remove_user, socket_status, start_socket,
            stop_socket,
        },
        heartbeat,
        intelli_app::latest_release,
        user::{update_user, user_data},
    },
    middlewares::auth_handler,
    states::AppState,
};

#[inline(always)]
pub(crate) fn api_routes(app_state: AppState) -> Router<AppState> {
    let auth_middleware = middleware::from_fn_with_state(app_state.clone(), auth_handler);

    let auth_router = Router::new()
        .route("/google/callback", get(callback))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout).route_layer(auth_middleware.clone()))
        .route("/refresh", get(refresh_token))
        .route("/verify/email", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password));

    let user_router = Router::new()
        .route("/", put(update_user))
        .route("/data", get(user_data))
        .route_layer(auth_middleware.clone());

    let app_router = Router::new().route("/releases/latest", get(latest_release));

    let websocket_router = Router::new()
        // .route("/championship/:id/socket", get(session_socket))
        // .route("/championship/:id/start", get(start_socket))
        .route("/championship/:id/status", get(socket_status))
        .route("/championship/:id/stop", get(stop_socket));

    let championship_router = Router::new()
        .route("/", post(create_championship))
        .route("/all", get(all_championships))
        .route("/:id", put(update_user))
        .route("/:id/user/add", put(update_user))
        .route("/:id/user/:user_id", delete(remove_user))
        .nest("/web_socket", websocket_router)
        .route_layer(auth_middleware.clone());

    Router::new()
        .route("/heartbeat", get(heartbeat))
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .nest("/championships", championship_router)
        .nest("/intelli-app", app_router)
}

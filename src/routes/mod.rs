use self::admin::admin_router;
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
        init,
        user::user_data,
    },
    middlewares::auth_handler,
    states::{AuthState, UserState},
};
use axum::{
    error_handling::HandleErrorLayer,
    middleware,
    routing::{get, post, IntoMakeService},
    Router,
};
use hyper::{Method, StatusCode};
use std::{sync::Arc, time::Duration};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, timeout::TimeoutLayer, ServiceBuilder};
use tower_http::cors::{AllowMethods, AllowOrigin, Any, CorsLayer};

mod admin;

// Handles Service Errors
async fn handle_error(e: Box<dyn std::error::Error + Send + Sync>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", e),
    )
}

#[inline(always)]
pub(crate) async fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    let auth_state = AuthState::new(&database);
    let user_state = Arc::new(UserState::new(&database).await);

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_headers(Any)
        .allow_methods(AllowMethods::list([Method::GET, Method::POST]));

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", get(refresh_token))
        .route("/verify/email", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route(
            "/logout",
            get(logout).route_layer(middleware::from_fn_with_state(
                user_state.clone(),
                auth_handler,
            )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(8, Duration::from_secs(120))),
        )
        .with_state(auth_state);

    let user_router = Router::new()
        .route("/data", get(user_data))
        .with_state(user_state.clone())
        .route_layer(middleware::from_fn_with_state(
            user_state.clone(),
            auth_handler,
        ));

    // todo: Delete Championship, Get Championship
    // todo: Add Round to Championship, Delete Round from Championship, Get Round from Championship and reference it to the corresponding session
    let championships_router = Router::new()
        .route("/", post(create_championship))
        .route("/:id", get(get_championship))
        .route("/all", get(all_championships))
        .route("/:id/start_socket", get(start_socket))
        .route("/:id/stop_socket", get(stop_socket))
        .route_layer(middleware::from_fn_with_state(
            user_state.clone(),
            auth_handler,
        ))
        .with_state(user_state.clone());

    Router::new()
        .route("/", get(init))
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .nest("/championships", championships_router)
        .nest("/admin", admin_router(user_state.clone()))
        .route(
            "/championships/:id/web_socket", // Removed /session/session:id to make it easier to use
            get(session_socket).with_state(user_state),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(TimeoutLayer::new(Duration::from_secs(5)))
                .layer(cors_layer),
        )
        .into_make_service()
}

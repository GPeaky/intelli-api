use crate::{
    config::Database,
    handlers::{
        auth::{forgot_password, login, logout, refresh_token, register, reset_password},
        championships::{
            active_sockets, create_championship, get_championship, start_socket, stop_socket,
        },
        init,
        user::user_data,
        verify::verify_email,
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
use hyper::{http::HeaderValue, Method, StatusCode};
use std::{sync::Arc, time::Duration};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, timeout::TimeoutLayer, ServiceBuilder};
use tower_http::cors::{AllowMethods, AllowOrigin, Any, CorsLayer};

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
    let user_state = UserState::new(&database).await;

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "https://intelli.gerardz.de",
        )))
        .allow_headers(Any)
        .allow_methods(AllowMethods::list([Method::GET, Method::POST]));

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", get(refresh_token))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route(
            "/logout",
            get(logout).route_layer(middleware::from_fn_with_state(
                user_state.clone(),
                auth_handler,
            )),
        )
        .with_state(auth_state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(8, Duration::from_secs(120))),
        );

    let user_router = Router::new()
        .route("/data", get(user_data))
        .with_state(user_state.clone())
        .route_layer(middleware::from_fn_with_state(
            user_state.clone(),
            auth_handler,
        ));

    let verify_router = Router::new()
        .route("/email", get(verify_email))
        .with_state(auth_state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(3, Duration::from_secs(30))),
        );

    // todo: Delete Championship, Get Championship
    // todo: Add Round to Championship, Delete Round from Championship, Get Round from Championship and reference it to the corresponding session
    let championships_router = Router::new()
        .route("/", post(create_championship))
        .route("/:id", get(get_championship))
        .route("/:id/start_socket", get(start_socket))
        .route("/:id/stop_socket", get(stop_socket))
        .route("/sockets", get(active_sockets))
        .route_layer(middleware::from_fn_with_state(
            user_state.clone(),
            auth_handler,
        ))
        .with_state(user_state);

    Router::new()
        .route("/", get(init))
        .nest("/auth", auth_router)
        .nest("/user", user_router)
        .nest("/verify", verify_router)
        .nest("/championships", championships_router)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(TimeoutLayer::new(Duration::from_secs(5))),
        )
        .layer(cors_layer)
        .into_make_service()
}

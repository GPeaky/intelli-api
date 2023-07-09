use crate::{
    config::Database,
    handlers::{
        auth::{login, logout, register},
        championships::create_championship,
        init,
        verify::verify_email,
    },
    middlewares::auth_handler,
    states::{AuthState, UserState},
};
use axum::{
    middleware,
    routing::{get, post, IntoMakeService},
    Router,
};
use std::sync::Arc;

pub(crate) async fn service_routes(database: Arc<Database>) -> IntoMakeService<Router> {
    let auth_state = AuthState::new(&database);
    let user_state = UserState::new(&database).await;

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/logout",
            get(logout).route_layer(middleware::from_fn_with_state(
                user_state.clone(),
                auth_handler,
            )),
        )
        .with_state(auth_state.clone());

    let verify_router = Router::new()
        .route("/email", get(verify_email))
        .with_state(auth_state);

    // todo: Delete Championship, Get Championship
    let championships_router = Router::new()
        .route("/", post(create_championship))
        .route_layer(middleware::from_fn_with_state(
            user_state.clone(),
            auth_handler,
        ))
        .with_state(user_state);

    // todo: Add Round to Championship, Delete Round from Championship, Get Round from Championship and reference it to the corresponding session

    Router::new()
        .route("/", get(init))
        .nest("/auth", auth_router)
        .nest("/verify", verify_router)
        .nest("/championships", championships_router)
        .into_make_service()
}

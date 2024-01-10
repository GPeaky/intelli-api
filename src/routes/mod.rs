use axum::Router;

use crate::states::AppState;

mod admin;
mod api;

pub fn routes(app_state: AppState) -> Router {
    // let admin_middleware = middleware::from_fn_with_state(app_state.clone(), admin_handler);

    Router::new()
        .nest("/", api::api_routes(app_state.clone()))
        .nest("/admin", admin::admin_routes())
        .with_state(app_state)
}

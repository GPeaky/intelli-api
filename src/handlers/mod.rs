use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod user;

pub(crate) async fn heartbeat() -> Response {
    StatusCode::OK.into_response()
}

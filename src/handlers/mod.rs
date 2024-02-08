use ntex::web::{HttpResponse, Responder};

pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod user;

pub(crate) async fn heartbeat() -> impl Responder {
    HttpResponse::Ok()
}

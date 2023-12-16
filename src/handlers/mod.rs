pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod intelli_app;
pub(crate) mod user;

use ntex::web;

pub(crate) async fn heartbeat() -> impl web::Responder {
    web::HttpResponse::Ok()
}

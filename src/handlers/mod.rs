use ntex::web;

pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod user;

pub(crate) async fn heartbeat() -> impl web::Responder {
    web::HttpResponse::Ok()
}

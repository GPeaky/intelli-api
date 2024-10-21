pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod user;

use ntex::web::HttpResponse;

pub(crate) async fn system_health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

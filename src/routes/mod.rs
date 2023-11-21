use crate::entity::UserExtension;
use ntex::web;
// mod api;

pub async fn handler_test(req: web::HttpRequest) -> impl web::Responder {
    let user = req.extensions().get::<UserExtension>().unwrap().clone();

    web::HttpResponse::Ok().json(&user)
}

#[inline(always)]
pub(crate) fn service_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::post().to(handler_test));
}

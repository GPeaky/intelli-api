use ntex::web;
// mod api;

pub async fn handler_test(req: web::HttpRequest) -> impl web::Responder {
    println!("{:?}", req);
    web::HttpResponse::Ok().body("Hello world!")
}

#[inline(always)]
pub(crate) fn service_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::post().to(handler_test));
}

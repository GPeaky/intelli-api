use ntex::{http::StatusCode, web};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Error deserializing entity from cache")]
    Deserialize,
    #[error("Error serializing entity to cache")]
    Serialize,
}

impl web::error::WebResponseError for CacheError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

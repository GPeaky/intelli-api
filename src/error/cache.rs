use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Error deserializing entity from cache")]
    Deserialize,
    #[error("Error serializing entity to cache")]
    Serialize,
}

impl WebResponseError for CacheError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

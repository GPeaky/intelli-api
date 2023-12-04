use ntex::{http::StatusCode, web};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("Socket not found")]
    NotFound,
    #[error("Socket already exists")]
    AlreadyExists,
    #[error("Socket is not active")]
    NotActive,
    #[error("Failed to send message")]
    FailedToSendMessage,
}

impl web::error::WebResponseError for SocketError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            SocketError::NotFound => StatusCode::BAD_REQUEST,
            SocketError::AlreadyExists => StatusCode::CONFLICT,
            SocketError::NotActive => StatusCode::NOT_FOUND,
            SocketError::FailedToSendMessage => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

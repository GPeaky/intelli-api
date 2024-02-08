use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum F123ServiceError {
    #[error("Error Opening UdpSocket")]
    UdpSocket,
    #[error("Unsupported Packet Format")]
    UnsupportedPacketFormat,
    #[allow(unused)]
    #[error("Not Online Session")]
    NotOnlineSession,
    #[error("Receiving data from udp socket")]
    ReceivingData,
    #[error("Error to encode packet data")]
    Encoding,
    #[error("Error to encode batched data")]
    BatchedEncoding,
    #[error("Error to compress batched data")]
    Compressing,
    #[error("Already Exists")]
    AlreadyExists,
    #[error("Service not active")]
    NotActive,
    #[error("Service not found")]
    NotFound,
}

impl WebResponseError for F123ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            F123ServiceError::UdpSocket => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::UnsupportedPacketFormat => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::NotOnlineSession => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::ReceivingData => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::Encoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::BatchedEncoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::Compressing => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::AlreadyExists => StatusCode::CONFLICT,
            F123ServiceError::NotActive => StatusCode::INTERNAL_SERVER_ERROR,
            F123ServiceError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

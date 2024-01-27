use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum F123Error {
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
}

impl WebResponseError for F123Error {
    fn status_code(&self) -> StatusCode {
        match self {
            F123Error::UdpSocket => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::UnsupportedPacketFormat => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::NotOnlineSession => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::ReceivingData => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::Encoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::BatchedEncoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::Compressing => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

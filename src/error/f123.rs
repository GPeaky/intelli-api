use ntex::{http::StatusCode, web};
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
}

impl web::error::WebResponseError for F123Error {
    fn status_code(&self) -> StatusCode {
        match self {
            F123Error::UdpSocket => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::UnsupportedPacketFormat => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::NotOnlineSession => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::ReceivingData => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::Encoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::BatchedEncoding => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }
}

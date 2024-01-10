use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::response::AppErrorResponse;

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

impl IntoResponse for F123Error {
    fn into_response(self) -> Response {
        let code = match self {
            F123Error::UdpSocket => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::UnsupportedPacketFormat => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::NotOnlineSession => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::ReceivingData => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::Encoding => StatusCode::INTERNAL_SERVER_ERROR,
            F123Error::BatchedEncoding => StatusCode::INTERNAL_SERVER_ERROR,
        };

        AppErrorResponse::send(code, Some(self.to_string()))
    }
}

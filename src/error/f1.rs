use ntex::http::StatusCode;

use super::AppError;

#[derive(Debug)]
pub enum F1ServiceError {
    UdpSocket,
    UnsupportedPacketFormat,
    #[allow(unused)]
    NotOnlineSession,
    ReceivingData,
    Encoding,
    BatchedEncoding,
    AlreadyExists,
    NotActive,
}

impl F1ServiceError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            F1ServiceError::UdpSocket => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::UnsupportedPacketFormat => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::NotOnlineSession => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::ReceivingData => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::Encoding => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::BatchedEncoding => StatusCode::INTERNAL_SERVER_ERROR,
            F1ServiceError::AlreadyExists => StatusCode::CONFLICT,
            F1ServiceError::NotActive => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            F1ServiceError::UdpSocket => "Error Opening UdpSocket",
            F1ServiceError::UnsupportedPacketFormat => "Unsupported Packet Format",
            F1ServiceError::NotOnlineSession => "Not Online Session",
            F1ServiceError::ReceivingData => "Receiving data from udp socket",
            F1ServiceError::Encoding => "Error to encode packet data",
            F1ServiceError::BatchedEncoding => "Error to encode batched data",
            F1ServiceError::AlreadyExists => "Already Exists",
            F1ServiceError::NotActive => "Service not active",
        }
    }
}

impl std::error::Error for F1ServiceError {}

impl std::fmt::Display for F1ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl From<F1ServiceError> for AppError {
    fn from(e: F1ServiceError) -> Self {
        AppError::F1(e)
    }
}

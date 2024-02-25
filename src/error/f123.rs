use ntex::http::StatusCode;

use super::AppError;

#[derive(Debug)]
pub enum F123ServiceError {
    UdpSocket,
    UnsupportedPacketFormat,
    #[allow(unused)]
    NotOnlineSession,
    ReceivingData,
    Encoding,
    BatchedEncoding,
    Compressing,
    AlreadyExists,
    NotActive,
    NotFound,
}

impl F123ServiceError {
    pub const fn status_code(&self) -> StatusCode {
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

    pub const fn error_message(&self) -> &'static str {
        match self {
            F123ServiceError::UdpSocket => "Error Opening UdpSocket",
            F123ServiceError::UnsupportedPacketFormat => "Unsupported Packet Format",
            F123ServiceError::NotOnlineSession => "Not Online Session",
            F123ServiceError::ReceivingData => "Receiving data from udp socket",
            F123ServiceError::Encoding => "Error to encode packet data",
            F123ServiceError::BatchedEncoding => "Error to encode batched data",
            F123ServiceError::Compressing => "Error to compress batched data",
            F123ServiceError::AlreadyExists => "Already Exists",
            F123ServiceError::NotActive => "Service not active",
            F123ServiceError::NotFound => "Service not found",
        }
    }
}

impl std::error::Error for F123ServiceError {}

impl std::fmt::Display for F123ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl From<F123ServiceError> for AppError {
    fn from(e: F123ServiceError) -> Self {
        AppError::F123(e)
    }
}

use ntex::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        StatusCode,
    },
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum DriverError {
    AlreadyExists,
}

impl DriverError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            DriverError::AlreadyExists => StatusCode::CONFLICT,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            DriverError::AlreadyExists => "Driver already exists",
        }
    }
}

impl std::error::Error for DriverError {}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl From<DriverError> for AppError {
    #[inline]
    fn from(e: DriverError) -> Self {
        AppError::Driver(e)
    }
}

// Added for middlewares
impl WebResponseError for DriverError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )
            .body(self.error_message())
    }
}

use ntex::{http::StatusCode, web};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User already exists")]
    AlreadyExists,
    #[error("User not found")]
    NotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Not verified user")]
    NotVerified,
    #[error("Use google to login")]
    GoogleLogin,
    #[error("Unauthorized user")]
    Unauthorized,
    #[error("Cannot Delete Yourself")]
    AutoDelete,
    #[error("User Already Active")]
    AlreadyActive,
    #[error("User is not active")]
    AlreadyInactive,
    #[error("Invalid Provider")]
    InvalidProvider,
    #[error("Using wrong provider")]
    WrongProvider,
}

impl web::error::WebResponseError for UserError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::NotVerified => StatusCode::UNAUTHORIZED,
            UserError::GoogleLogin => StatusCode::BAD_REQUEST,
            UserError::Unauthorized => StatusCode::UNAUTHORIZED,
            UserError::AutoDelete => StatusCode::BAD_REQUEST,
            UserError::AlreadyActive => StatusCode::BAD_REQUEST,
            UserError::AlreadyInactive => StatusCode::BAD_REQUEST,
            UserError::InvalidProvider => StatusCode::BAD_REQUEST,
            UserError::WrongProvider => StatusCode::BAD_REQUEST,
        }
    }
}

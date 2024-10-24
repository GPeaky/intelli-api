use ntex::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        StatusCode,
    },
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};

use super::AppError;

#[derive(Debug)]
pub enum UserError {
    AlreadyExists,
    NotFound,
    InvalidCredentials,
    NotVerified,
    DiscordAuth,
    Unauthorized,
    SelfDelete,
    AlreadyActive,
    AlreadyInactive,
    WrongProvider,
    InvalidUpdate,
}

impl UserError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            UserError::AlreadyExists => StatusCode::CONFLICT,
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UserError::NotVerified => StatusCode::UNAUTHORIZED,
            UserError::DiscordAuth => StatusCode::BAD_REQUEST,
            UserError::Unauthorized => StatusCode::UNAUTHORIZED,
            UserError::SelfDelete => StatusCode::BAD_REQUEST,
            UserError::AlreadyActive => StatusCode::BAD_REQUEST,
            UserError::AlreadyInactive => StatusCode::BAD_REQUEST,
            UserError::WrongProvider => StatusCode::BAD_REQUEST,
            UserError::InvalidUpdate => StatusCode::BAD_REQUEST,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            UserError::AlreadyExists => "User already exists",
            UserError::NotFound => "User not found",
            UserError::InvalidCredentials => "Invalid credentials",
            UserError::NotVerified => "Not verified user",
            UserError::DiscordAuth => "Use discord auth",
            UserError::Unauthorized => "Unauthorized user",
            UserError::SelfDelete => "Cannot Delete Yourself",
            UserError::AlreadyActive => "User Already Active",
            UserError::AlreadyInactive => "User is not active",
            UserError::WrongProvider => "Using wrong provider",
            UserError::InvalidUpdate => "Invalid Update",
        }
    }
}

impl std::error::Error for UserError {}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl From<UserError> for AppError {
    #[inline]
    fn from(e: UserError) -> Self {
        AppError::User(e)
    }
}

// Added for middlewares
impl WebResponseError for UserError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )
            .body(self.error_message())
    }
}

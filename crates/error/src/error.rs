use deadpool_postgres::{tokio_postgres::Error as PgError, PoolError};
use ntex::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        StatusCode,
    },
    rt::JoinError,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
};
use tracing::error;

pub use championship::*;
pub use common::*;
pub use driver::*;
pub use f1::*;
pub use firewall::*;
pub use token::*;
pub use user::*;

mod championship;
mod common;
mod driver;
mod f1;
mod firewall;
mod token;
mod user;

pub type AppResult<T> = Result<T, AppError>;

// TODO: Handle 3rd party errors in a better way
#[derive(Debug)]
pub enum AppError {
    User(UserError),
    Championship(ChampionshipError),
    Driver(DriverError),
    Token(TokenError),
    Common(CommonError),
    F1(F1ServiceError),
    Firewall(FirewallError),
    Twilight,
    PgError,
    PgPool,
    Reqwest,
    Sailfish,
}

impl From<JoinError> for AppError {
    fn from(value: JoinError) -> Self {
        error!("Join Error: {}", value);
        AppError::Common(CommonError::InternalServerError)
    }
}

impl From<PgError> for AppError {
    fn from(value: PgError) -> Self {
        error!("PgError: {}", value);
        AppError::PgError
    }
}

impl From<PoolError> for AppError {
    fn from(value: PoolError) -> Self {
        error!("PgPool Error: {:?}", value);
        AppError::PgPool
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        error!("ReqwestError: {}", value);
        AppError::Reqwest
    }
}

impl From<sailfish::RenderError> for AppError {
    fn from(value: sailfish::RenderError) -> Self {
        error!("Sailfish Error: {:?}", value);
        AppError::Sailfish
    }
}

impl From<twilight_http::Error> for AppError {
    fn from(value: twilight_http::Error) -> Self {
        error!("Twilight error: {:?}", value);
        AppError::Twilight
    }
}

impl From<twilight_http::response::DeserializeBodyError> for AppError {
    fn from(value: twilight_http::response::DeserializeBodyError) -> Self {
        error!("Twilight Deserialize error: {:?}", value);
        AppError::Twilight
    }
}

impl AppError {
    const fn error_status(&self) -> StatusCode {
        match self {
            AppError::User(e) => e.status_code(),
            AppError::Championship(e) => e.status_code(),
            AppError::Driver(e) => e.status_code(),
            AppError::Token(e) => e.status_code(),
            AppError::Common(e) => e.status_code(),
            AppError::F1(e) => e.status_code(),
            AppError::Firewall(e) => e.status_code(),
            AppError::PgError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PgPool => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Reqwest => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sailfish => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Twilight => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    const fn error_message(&self) -> &'static str {
        match self {
            AppError::User(e) => e.error_message(),
            AppError::Championship(e) => e.error_message(),
            AppError::Driver(e) => e.error_message(),
            AppError::Token(e) => e.error_message(),
            AppError::Common(e) => e.error_message(),
            AppError::F1(e) => e.error_message(),
            AppError::Firewall(e) => e.error_message(),
            AppError::PgError => "Database error",
            AppError::PgPool => "Pool error",
            AppError::Reqwest => "Reqwest error",
            AppError::Sailfish => "Email Render Error",
            AppError::Twilight => "Twilight error",
        }
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl WebResponseError for AppError {
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(self.error_status())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )
            .body(self.error_message())
    }
}

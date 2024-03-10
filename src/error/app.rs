use deadpool_postgres::{tokio_postgres::Error as PgError, PoolError};
use deadpool_redis::{redis::RedisError, PoolError as RedisPoolError};
use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
    ws::error::HandshakeError,
};
use tracing::error;

use super::{
    user::UserError, CacheError, ChampionshipError, CommonError, F123ServiceError, FirewallError,
    TokenError,
};

pub type AppResult<T> = Result<T, AppError>;

// Todo: Handle 3rd party errors in a better way
#[derive(Debug)]
pub enum AppError {
    User(UserError),
    Championship(ChampionshipError),
    Token(TokenError),
    Common(CommonError),
    Cache(CacheError),
    F123(F123ServiceError),
    Firewall(FirewallError),
    PgError(PgError),
    PgPool,
    Redis,
    RedisPool,
    Handshake(HandshakeError),
    Reqwest(reqwest::Error),
    Sailfish,
}

impl From<PgError> for AppError {
    fn from(value: PgError) -> Self {
        AppError::PgError(value)
    }
}

impl From<PoolError> for AppError {
    fn from(value: PoolError) -> Self {
        error!("PgPool Error: {:?}", value);
        AppError::PgPool
    }
}

impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        error!("Redis Error: {:?}", value);
        AppError::Redis
    }
}

impl From<RedisPoolError> for AppError {
    fn from(_: RedisPoolError) -> Self {
        AppError::RedisPool
    }
}

impl From<HandshakeError> for AppError {
    fn from(value: HandshakeError) -> Self {
        AppError::Handshake(value)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::Reqwest(value)
    }
}

impl From<sailfish::RenderError> for AppError {
    fn from(value: sailfish::RenderError) -> Self {
        error!("Sailfish Error: {:?}", value);
        AppError::Sailfish
    }
}

impl AppError {
    const fn error_status(&self) -> StatusCode {
        match self {
            AppError::User(e) => e.status_code(),
            AppError::Championship(e) => e.status_code(),
            AppError::Token(e) => e.status_code(),
            AppError::Common(e) => e.status_code(),
            AppError::Cache(e) => e.status_code(),
            AppError::F123(e) => e.status_code(),
            AppError::Firewall(e) => e.status_code(),
            AppError::PgError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PgPool => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisPool => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Handshake(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sailfish => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    const fn error_message(&self) -> &'static str {
        match self {
            AppError::User(e) => e.error_message(),
            AppError::Championship(e) => e.error_message(),
            AppError::Token(e) => e.error_message(),
            AppError::Common(e) => e.error_message(),
            AppError::Cache(e) => e.error_message(),
            AppError::F123(e) => e.error_message(),
            AppError::Firewall(e) => e.error_message(),
            AppError::PgError(_) => "Database error",
            AppError::PgPool => "Pool error",
            AppError::Redis => "Cache error",
            AppError::RedisPool => "Cache pool error",
            AppError::Handshake(_) => "Handshake error",
            AppError::Reqwest(_) => "Reqwest error",
            AppError::Sailfish => "Email Render Error",
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
            .set_header("content-type", "text/plain; charset=utf-8")
            .body(self.error_message())
    }
}

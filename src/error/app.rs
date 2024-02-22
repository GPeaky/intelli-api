use deadpool_postgres::{tokio_postgres::Error as PgError, PoolError};
use deadpool_redis::{redis::RedisError, PoolError as RedisPoolError};
use lettre::Message;
use ntex::{
    http::StatusCode,
    web::{error::WebResponseError, HttpRequest, HttpResponse},
    ws::error::HandshakeError,
};
use thiserror::Error;
use tracing::error;

use super::{
    user::UserError, CacheError, ChampionshipError, CommonError, F123ServiceError, TokenError,
};

pub type AppResult<T> = Result<T, AppError>;

// TODO: Add more errors and handle them in a better way
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Championship(#[from] ChampionshipError),
    #[error(transparent)]
    Token(#[from] TokenError),
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error(transparent)]
    Cache(#[from] CacheError),
    #[error(transparent)]
    F123(#[from] F123ServiceError),
    #[error(transparent)]
    PgError(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PoolError),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    RedisPool(#[from] RedisPoolError),
    #[error(transparent)]
    Handshake(#[from] HandshakeError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Sailfish(#[from] sailfish::RenderError),
    #[error(transparent)]
    Lettre(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    MessageSender(#[from] loole::SendError<Message>),
}

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::User(e) => e.status_code(),
            AppError::Championship(e) => e.status_code(),
            AppError::Token(e) => e.status_code(),
            AppError::Common(e) => e.status_code(),
            AppError::Cache(e) => e.status_code(),
            AppError::F123(e) => e.status_code(),
            AppError::PgError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PgPool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisPool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Handshake(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sailfish(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Lettre(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MessageSender(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, r: &HttpRequest) -> HttpResponse {
        match self {
            AppError::User(e) => e.error_response(r),
            AppError::Championship(e) => e.error_response(r),
            AppError::Token(e) => e.error_response(r),
            AppError::Common(e) => e.error_response(r),
            AppError::Cache(e) => e.error_response(r),
            AppError::F123(e) => e.error_response(r),
            AppError::PgError(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Database error")
            }

            AppError::PgPool(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Pool error")
            }

            AppError::Redis(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Cache error")
            }

            AppError::RedisPool(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Cache pool error")
            }

            AppError::Handshake(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Handshake error")
            }

            AppError::Reqwest(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Reqwest error")
            }

            AppError::Sailfish(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Email Render Error")
            }

            AppError::Lettre(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Email Error")
            }
            AppError::MessageSender(e) => {
                error!("{e}");

                HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Email Error")
            }
        }
    }
}

use super::{user::UserError, CacheError, ChampionshipError, CommonError, SocketError, TokenError};
use deadpool_postgres::{tokio_postgres::Error as PgError, PoolError};
use deadpool_redis::{redis::RedisError, PoolError as RedisPoolError};
use log::error;
use ntex::{http::StatusCode, web, ws::error::HandshakeError};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

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
    Socket(#[from] SocketError),
    #[error(transparent)]
    Database(#[from] PgError),
    #[error(transparent)]
    DbPool(#[from] PoolError),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    RedisPool(#[from] RedisPoolError),
    #[error(transparent)]
    Handshake(#[from] HandshakeError),
}

// TODO: Handle Database, Redis and Pool errors in a better way
impl web::error::WebResponseError for AppError {
    fn error_response(&self, r: &web::HttpRequest) -> web::HttpResponse {
        match self {
            AppError::User(e) => e.error_response(r),
            AppError::Championship(e) => e.error_response(r),
            AppError::Token(e) => e.error_response(r),
            AppError::Common(e) => e.error_response(r),
            AppError::Cache(e) => e.error_response(r),
            AppError::Socket(e) => e.error_response(r),
            AppError::Database(e) => {
                error!("{e}");

                web::HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Database error")
            }

            AppError::DbPool(e) => {
                error!("{e}");

                web::HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Pool error")
            }

            AppError::Redis(e) => {
                error!("{e}");

                web::HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Cache error")
            }

            AppError::RedisPool(e) => {
                error!("{e}");

                web::HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Cache pool error")
            }

            AppError::Handshake(e) => {
                error!("{e}");

                web::HttpResponse::build(self.status_code())
                    .set_header("content-type", "text/html; charset=utf-8")
                    .body("Handshake error")
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::User(e) => e.status_code(),
            AppError::Championship(e) => e.status_code(),
            AppError::Token(e) => e.status_code(),
            AppError::Common(e) => e.status_code(),
            AppError::Cache(e) => e.status_code(),
            AppError::Socket(e) => e.status_code(),
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DbPool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisPool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Handshake(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

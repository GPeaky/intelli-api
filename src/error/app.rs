use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::BcryptError;
use deadpool_postgres::{tokio_postgres::Error as PgError, PoolError};
use deadpool_redis::{redis::RedisError, PoolError as RedisPoolError};
use thiserror::Error;
use tracing::error;

use crate::response::AppErrorResponse;

use super::{
    user::UserError, CacheError, ChampionshipError, CommonError, F123Error, SocketError, TokenError,
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
    Socket(#[from] SocketError),
    #[error(transparent)]
    F123(#[from] F123Error),
    #[error(transparent)]
    PgError(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PoolError),
    #[error(transparent)]
    Bcrypt(#[from] BcryptError),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    RedisPool(#[from] RedisPoolError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Sailfish(#[from] sailfish::RenderError),
    #[error(transparent)]
    Lettre(#[from] lettre::transport::smtp::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::User(e) => e.into_response(),
            AppError::Championship(e) => e.into_response(),
            AppError::Token(e) => e.into_response(),
            AppError::Common(e) => e.into_response(),
            AppError::Cache(e) => e.into_response(),
            AppError::Socket(e) => e.into_response(),
            AppError::F123(e) => e.into_response(),
            AppError::PgError(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Pg Error".to_string()),
                )
            }

            AppError::PgPool(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Pool Error".to_string()),
                )
            }

            AppError::Bcrypt(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Bcrypt Error".to_string()),
                )
            }

            AppError::Redis(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Redis Error".to_string()),
                )
            }

            AppError::RedisPool(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Cache Pool Error".to_string()),
                )
            }

            AppError::Reqwest(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Reqwest Error".to_string()),
                )
            }

            AppError::Sailfish(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Email Render Error".to_string()),
                )
            }

            AppError::Lettre(e) => {
                error!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Email Error".to_string()),
                )
            }
        }
    }
}

// impl WebResponseError for AppError {
//     #[inline(always)]
//     fn status_code(&self) -> StatusCode {
//
//     }

//     fn error_response(&self, r: &HttpRequest) -> HttpResponse {
//         match self {
//             AppError::User(e) => e.into_response(),
//             AppError::Championship(e) => e.error_response(r),
//             AppError::Token(e) => e.error_response(r),
//             AppError::Common(e) => e.error_response(r),
//             AppError::Cache(e) => e.error_response(r),
//             AppError::Socket(e) => e.error_response(r),
//             AppError::F123(e) => e.error_response(r),
//             AppError::PgError(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Database error")
//             }

//             AppError::PgPool(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Pool error")
//             }

//             AppError::Bcrypt(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Encryption error")
//             }

//             AppError::Redis(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Cache error")
//             }

//             AppError::RedisPool(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Cache pool error")
//             }

//             AppError::Handshake(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Handshake error")
//             }

//             AppError::Reqwest(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Reqwest error")
//             }

//             AppError::Sailfish(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Email Render Error")
//             }

//             AppError::Lettre(e) => {
//                 error!("{e}");

//                 HttpResponse::build(self.status_code())
//                     .set_header("content-type", "text/html; charset=utf-8")
//                     .body("Email Error")
//             }
//         }
//     }
// }

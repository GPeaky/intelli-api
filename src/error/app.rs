use super::{user::UserError, CacheError, ChampionshipError, CommonError, SocketError, TokenError};
use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bb8_redis::{bb8::RunError, redis::RedisError};
use thiserror::Error;
use tracing::info;

pub type AppResult<T> = Result<T, AppError>;

// TODO: Handle Database, Redis and Pool errors in a better way
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
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Pool(#[from] RunError<RedisError>),
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
            AppError::Database(e) => {
                info!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Database Error".to_owned()),
                )
            }

            AppError::Redis(e) => {
                info!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Cache Error".to_owned()),
                )
            }

            AppError::Pool(e) => {
                info!("{e}");

                AppErrorResponse::send(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("Cache Error".to_owned()),
                )
            }
        }
    }
}

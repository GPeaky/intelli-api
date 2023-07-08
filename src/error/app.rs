use super::{user::UserError, ChampionshipError, CommonError, TokenError};
use crate::response::AppErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use scylla::transport::{
    errors::{DbError, QueryError},
    query_result::{RowsExpectedError, SingleRowTypedError},
};
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
    Query(#[from] QueryError),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    RowsExpected(#[from] RowsExpectedError),
    #[error(transparent)]
    SingleRowTyped(#[from] SingleRowTypedError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::User(e) => e.into_response(),
            AppError::Championship(e) => e.into_response(),
            AppError::Token(e) => e.into_response(),
            AppError::Common(e) => e.into_response(),
            AppError::Query(e) => {
                AppErrorResponse::send(StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }

            AppError::Db(e) => {
                AppErrorResponse::send(StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }

            AppError::RowsExpected(e) => {
                AppErrorResponse::send(StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }

            AppError::SingleRowTyped(e) => {
                AppErrorResponse::send(StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
        }
    }
}

use ntex::http::StatusCode;
use std::{error::Error, fmt::Display};

use super::AppError;

#[derive(Debug)]
pub enum CacheError {
    Deserialize,
    Serialize,
}

impl CacheError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            CacheError::Deserialize => StatusCode::INTERNAL_SERVER_ERROR,
            CacheError::Serialize => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            CacheError::Deserialize => "Error deserializing entity from cache",
            CacheError::Serialize => "Error serializing entity to cache",
        }
    }
}

impl Error for CacheError {}

impl From<CacheError> for AppError {
    fn from(e: CacheError) -> Self {
        AppError::Cache(e)
    }
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

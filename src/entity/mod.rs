mod championship;
mod saved_sessions;
mod user;

use crate::error::AppResult;
pub use championship::*;
use deadpool_postgres::tokio_postgres::Row;
#[allow(unused)]
pub use saved_sessions::*;
pub use user::*;

pub trait FromRow {
    fn from_row(row: &Row) -> AppResult<Self>
    where
        Self: std::marker::Sized;
}

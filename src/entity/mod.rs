mod championship;
mod saved_sessions;
mod user;

use crate::error::AppResult;
use bb8_postgres::tokio_postgres::Row;
pub use championship::*;
#[allow(unused)]
pub use saved_sessions::*;
pub use user::*;

pub trait FromRow {
    fn from_row<'a>(row: &'a Row) -> AppResult<Self>
    where
        Self: std::marker::Sized;
}

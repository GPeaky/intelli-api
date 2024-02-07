use deadpool_postgres::tokio_postgres::Row;

pub use championship::*;
pub use user::*;

use crate::error::AppResult;

mod championship;
mod saved_sessions;
mod user;

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> AppResult<Self>;
}

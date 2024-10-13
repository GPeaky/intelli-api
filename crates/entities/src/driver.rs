use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;

pub type SharedDriver = Arc<Driver>;

/// Represents a driver in the championship
#[allow(unused)]
pub struct Driver {
    pub steam_name: Box<str>,
    pub nationality: i16,
    pub user_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Driver {
    /// Creates a Driver from a database row
    #[inline]
    pub fn from_row(row: &Row) -> Self {
        Driver {
            steam_name: row.get(0),
            nationality: row.get(1),
            user_id: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        }
    }

    /// Creates an Arc<Driver> from a database row
    #[inline]
    pub fn from_row_arc(row: &Row) -> Arc<Self> {
        Arc::new(Driver::from_row(row))
    }
}

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;

#[allow(unused)]
pub struct Driver {
    id: i32,
    steam_name: String,
    nationality: i16,
    user_id: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl Driver {
    #[inline]
    #[allow(unused)]
    pub fn from_row(row: &Row) -> Self {
        Driver {
            id: row.get(0),
            steam_name: row.get(1),
            nationality: row.get(2),
            user_id: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
        }
    }
}

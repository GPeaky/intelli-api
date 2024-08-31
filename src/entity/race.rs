use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;

#[allow(unused)]
pub struct Race {
    id: i32,
    championship_id: i32,
    name: String,
    date: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Race {
    #[inline]
    #[allow(unused)]
    pub fn from_row(row: &Row) -> Self {
        Race {
            id: row.get(0),
            championship_id: row.get(1),
            name: row.get(2),
            date: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
        }
    }
}

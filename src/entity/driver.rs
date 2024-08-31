use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;

#[allow(unused)]
pub struct Driver {
    steam_name: String,
    discord_id: i64,
    nationality: i8,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Driver {
    #[inline]
    #[allow(unused)]
    pub fn from_row(row: &Row) -> Self {
        Driver {
            steam_name: row.get(0),
            discord_id: row.get(1),
            nationality: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        }
    }
}

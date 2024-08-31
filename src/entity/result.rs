use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;

#[allow(unused)]
pub struct Result {
    race_id: i32,
    session_type: i16,
    data: Vec<u8>,
    create_at: DateTime<Utc>,
}

impl Result {
    #[inline]
    #[allow(unused)]
    pub fn from_row(row: &Row) -> Self {
        Result {
            race_id: row.get(0),
            session_type: row.get(1),
            data: row.get(2),
            create_at: row.get(3),
        }
    }
}

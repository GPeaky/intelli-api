use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct EventData {
    pub id: i32,
    pub session_id: i64,
    pub string_code: String,
    pub events: Vec<u8>,
}

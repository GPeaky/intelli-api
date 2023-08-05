use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct EventData {
    pub session_id: i64,
    pub string_code: String,
    pub events: Vec<Vec<u8>>,
}

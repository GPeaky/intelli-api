use chrono::{DateTime, Utc};
use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Championship {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub port: i16,
    pub updated_at: DateTime<Utc>,
    pub user_id: i32,
}

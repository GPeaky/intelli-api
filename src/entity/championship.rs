use chrono::{DateTime, Utc};
use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Championship {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub port: i32,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
}

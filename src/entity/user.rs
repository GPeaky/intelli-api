use chrono::{DateTime, Utc};
use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password: String,
    pub updated_at: DateTime<Utc>,
    pub username: String,
}

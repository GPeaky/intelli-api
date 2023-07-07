use chrono::{DateTime, Utc};
use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub password: String,
    pub updated_at: DateTime<Utc>,
    pub username: String,
}

use chrono::{DateTime, Utc};
use sqlx::{Type, FromRow};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Type, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: Role,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

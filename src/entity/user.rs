use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[repr(u8)]
#[derive(Type, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub avatar: String,
    pub role: Role,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

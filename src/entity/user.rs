use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::sync::Arc;

pub type UserExtension = Arc<User>;

#[repr(u8)]
#[derive(Type, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provider {
    Local,
    Google,
}

#[repr(u8)]
#[derive(Type, Debug, Serialize, PartialEq, Eq)]
pub enum Role {
    Free,
    Premium,
    Admin,
}

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(skip_serializing)]
    pub provider: Provider,
    pub avatar: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub active: bool,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

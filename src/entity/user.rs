use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::sync::Arc;

pub type UserExtension = Arc<User>;

#[derive(Type, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "provider")]
pub enum Provider {
    Local,
    Google,
}

#[derive(Type, Debug, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "role")]
pub enum Role {
    Free,
    Premium,
    Business,
    Admin,
}

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: i32,
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

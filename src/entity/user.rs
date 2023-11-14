use chrono::{DateTime, Utc};
use rkyv::{Archive, Deserialize as RDeserialize, Serialize as RSerialize};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::sync::Arc;

pub type UserExtension = Arc<User>;

#[derive(Type, Debug, Archive, RDeserialize, RSerialize, Serialize, Deserialize, PartialEq, Eq)]
#[archive(check_bytes)]
#[sqlx(type_name = "provider")]
pub enum Provider {
    Local,
    Google,
}

#[derive(Type, Debug, Archive, RDeserialize, RSerialize, Serialize, PartialEq, Eq)]
#[archive(check_bytes)]
#[sqlx(type_name = "role")]
pub enum Role {
    Free,
    Premium,
    Business,
    Admin,
}

#[derive(Debug, Serialize, Archive, RDeserialize, RSerialize, FromRow)]
#[archive(check_bytes)]
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

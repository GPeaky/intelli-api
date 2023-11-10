use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[repr(u8)]
#[derive(Type, Debug, Serialize, Deserialize)]
pub enum Category {
    F1,
    F2,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Championship {
    pub id: u32,
    pub port: u16,
    pub name: String,
    pub category: Category,
    pub season: u16,
    pub driver_count: u8,
    pub owner_id: u32,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

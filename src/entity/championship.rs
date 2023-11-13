use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "category")]
pub enum Category {
    F1,
    F2,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Championship {
    pub id: i32,
    pub port: i32,
    pub name: String,
    pub category: Category,
    pub season: i16,
    pub driver_count: i16,
    pub owner_id: i32,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

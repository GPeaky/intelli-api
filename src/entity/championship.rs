use chrono::{DateTime, Utc};
use rkyv::{Archive, Deserialize as RDeserialize, Serialize as RSerialize};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Type, Debug, Archive, RDeserialize, RSerialize, Serialize, Deserialize)]
#[sqlx(type_name = "category")]
#[archive(check_bytes)]
pub enum Category {
    F1,
    F2,
}

#[derive(Debug, Serialize, Archive, RDeserialize, RSerialize, FromRow)]
#[archive(check_bytes)]
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

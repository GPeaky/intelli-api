use super::FromRow;
use crate::error::AppResult;
use bb8_postgres::tokio_postgres::Row;
use chrono::{DateTime, Utc};
use postgres_types::FromSql;
use rkyv::{Archive, Deserialize as RDeserialize, Serialize as RSerialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Archive, RDeserialize, RSerialize, Serialize, Deserialize, FromSql)]
#[postgres(name = "category")]
#[archive(check_bytes)]
pub enum Category {
    #[postgres(name = "F1")]
    F1,
    #[postgres(name = "F2")]
    F2,
}

#[derive(Debug, Serialize, Archive, RDeserialize, RSerialize)]
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

impl FromRow for Championship {
    fn from_row<'a>(row: &'a Row) -> AppResult<Self> {
        Ok(Championship {
            id: row.try_get("id")?,
            port: row.try_get("port")?,
            name: row.try_get("name")?,
            category: row.try_get("category")?,
            season: row.try_get("season")?,
            driver_count: row.try_get("driver_count")?,
            owner_id: row.try_get("owner_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

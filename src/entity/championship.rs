use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use postgres_derive::{FromSql, ToSql};
use rkyv::{Archive, Deserialize as RDeserialize, Serialize as RSerialize};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Archive, RDeserialize, RSerialize, Serialize, Deserialize, FromSql, ToSql)]
#[postgres(name = "championship_category")]
pub enum Category {
    #[postgres(name = "F1")]
    F1,
    #[postgres(name = "F2")]
    F2,
}

#[derive(Debug, Serialize, Archive, RDeserialize, RSerialize)]
pub struct Championship {
    pub id: i32,
    pub port: i32,
    pub name: String,
    pub category: Category,
    pub season: i16,
    pub driver_count: i16,
    pub owner_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Championship {
    pub fn try_from_rows(rows: &Vec<Row>) -> AppResult<Vec<Championship>> {
        let mut championships = Vec::with_capacity(rows.len());

        for row in rows {
            championships.push(Championship::try_from(row)?);
        }

        Ok(championships)
    }
}

impl TryFrom<&Row> for Championship {
    type Error = AppError;

    #[inline]
    fn try_from(value: &Row) -> AppResult<Championship> {
        Ok(Championship {
            id: value.try_get("id")?,
            port: value.try_get("port")?,
            name: value.try_get("name")?,
            category: value.try_get("category")?,
            season: value.try_get("season")?,
            driver_count: value.try_get("driver_count")?,
            owner_id: value.try_get("owner_id")?,
            created_at: value.try_get("created_at")?,
            updated_at: value.try_get("updated_at")?,
        })
    }
}

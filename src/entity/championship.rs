use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::{Row, RowStream};
use postgres_derive::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, FromSql, ToSql)]
#[postgres(name = "championship_category")]
pub enum Category {
    #[postgres(name = "F1")]
    F1,
    #[postgres(name = "F2")]
    F2,
}

#[derive(Debug, Serialize)]
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
    #[inline(always)]
    pub fn from_row(row: &Row) -> Arc<Championship> {
        Arc::new(Championship::from(row))
    }

    #[inline]
    pub async fn from_row_stream(it: RowStream) -> AppResult<Vec<Arc<Championship>>> {
        tokio::pin!(it);
        let mut championships = Vec::with_capacity(it.rows_affected().unwrap_or(0) as usize);

        while let Some(row) = it.try_next().await? {
            championships.push(Championship::from_row(&row))
        }

        Ok(championships)
    }
}

impl From<&Row> for Championship {
    #[inline]
    fn from(value: &Row) -> Self {
        Self {
            id: value.get(0),
            port: value.get(1),
            name: value.get(2),
            category: value.get(3),
            season: value.get(4),
            driver_count: value.get(5),
            owner_id: value.get(6),
            created_at: value.get(7),
            updated_at: value.get(8),
        }
    }
}

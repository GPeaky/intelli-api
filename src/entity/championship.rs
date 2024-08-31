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
    pub owner_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Championship {
    #[inline]
    pub fn from_row(row: &Row) -> Self {
        Championship {
            id: row.get(0),
            port: row.get(1),
            name: row.get(2),
            category: row.get(3),
            owner_id: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        }
    }

    #[inline]
    pub fn from_row_arc(row: &Row) -> Arc<Self> {
        Arc::new(Championship::from_row(row))
    }

    #[inline]
    pub async fn from_row_stream(it: RowStream) -> AppResult<Vec<Arc<Self>>> {
        tokio::pin!(it);
        let mut championships = Vec::with_capacity(it.rows_affected().unwrap_or(0) as usize);

        while let Some(row) = it.try_next().await? {
            championships.push(Championship::from_row_arc(&row))
        }

        Ok(championships)
    }
}

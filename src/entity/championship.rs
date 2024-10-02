use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::{Row, RowStream};
use postgres_derive::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::error::AppResult;

/// Shared reference to a User
pub type SharedChampionship = Arc<Championship>;

/// Championship roles
#[derive(Debug, Default, Serialize, Deserialize, FromSql, ToSql, PartialEq)]
#[postgres(name = "championship_role")]
pub enum ChampionshipRole {
    #[default]
    #[postgres(name = "Visitor")]
    Visitor,
    #[postgres(name = "Engineer")]
    Engineer,
    #[postgres(name = "Admin")]
    Admin,
}

/// Championship categories
#[derive(Debug, Serialize, Deserialize, FromSql, ToSql)]
#[postgres(name = "championship_category")]
pub enum Category {
    #[postgres(name = "F1")]
    F1,
    #[postgres(name = "F2")]
    F2,
}

pub struct ChampionshipRelation {
    pub role: ChampionshipRole,
    pub team_id: Option<i16>,
}

/// Represents a championship
#[derive(Debug, Serialize)]
pub struct Championship {
    pub id: i32,
    pub port: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub owner_id: i32,
    pub category: Category,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Championship {
    /// Creates a Championship from a database row
    #[inline]
    pub fn from_row(row: &Row) -> Self {
        Championship {
            id: row.get(0),
            port: row.get(1),
            name: row.get(2),
            owner_id: row.get(3),
            category: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        }
    }

    /// Creates an Arc<Championship> from a database row
    #[inline]
    pub fn from_row_arc(row: &Row) -> Arc<Self> {
        Arc::new(Championship::from_row(row))
    }

    /// Creates a Vec<Arc<Championship>> from a RowStream
    #[inline]
    pub async fn from_row_stream(it: RowStream) -> AppResult<Vec<Arc<Self>>> {
        tokio::pin!(it);
        let mut championships = Vec::new();

        while let Some(row) = it.try_next().await? {
            championships.push(Championship::from_row_arc(&row))
        }

        Ok(championships)
    }
}

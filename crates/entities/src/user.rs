use std::sync::Arc;

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use ntex::web::HttpRequest;
use postgres_derive::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

use error::{AppError, AppResult, CommonError};

/// Shared reference to a User
pub type SharedUser = Arc<User>;

/// User authentication provider
#[derive(Debug, Serialize, Deserialize, PartialEq, FromSql, ToSql)]
#[postgres(name = "user_provider")]
pub enum Provider {
    #[postgres(name = "Local")]
    Local,
    #[postgres(name = "Discord")]
    Discord,
}

/// User role in the system
#[derive(Debug, Clone, Copy, Serialize, PartialEq, FromSql, ToSql)]
#[postgres(name = "user_role")]
pub enum Role {
    #[postgres(name = "User")]
    User,
    #[postgres(name = "Premium")]
    Premium,
    #[postgres(name = "Admin")]
    Admin,
}

/// Represents a user in the system
#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub email: Box<str>,
    pub username: Box<str>,
    #[serde(skip_serializing)]
    pub password: Option<Box<str>>,
    pub avatar: Box<str>,
    #[serde(skip_serializing)]
    pub provider: Provider,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<i64>,
    #[serde(skip_serializing)]
    pub active: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    /// Creates a User from a database row
    #[inline]
    pub fn from_row(row: &Row) -> Self {
        User {
            id: row.get(0),
            email: row.get(1),
            username: row.get(2),
            password: row.get(3),
            avatar: row.get(4),
            provider: row.get(5),
            role: row.get(6),
            discord_id: row.get(7),
            active: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        }
    }

    /// Creates an Arc<User> from a database row
    #[inline]
    pub fn from_row_arc(row: &Row) -> Arc<Self> {
        Arc::new(User::from_row(row))
    }
}

/// Extension trait for extracting user information from HttpRequest
pub trait UserExtension {
    fn user(&self) -> AppResult<SharedUser>;
    fn user_id(&self) -> AppResult<i32>;
}

impl UserExtension for HttpRequest {
    /// Retrieves the SharedUser from the request extensions
    #[inline]
    fn user(&self) -> AppResult<SharedUser> {
        self.extensions()
            .get::<SharedUser>()
            .cloned()
            .ok_or(AppError::Common(CommonError::InternalServerError))
    }

    /// Retrieves the user ID from the request extensions
    #[inline]
    fn user_id(&self) -> AppResult<i32> {
        self.extensions()
            .get::<SharedUser>()
            .map(|user| user.id)
            .ok_or(AppError::Common(CommonError::InternalServerError))
    }
}

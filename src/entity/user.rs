use chrono::{DateTime, Utc};
use scylla::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub otp_auth_url: Option<String>,
    #[serde(skip_serializing)]
    pub otp_base32: Option<String>,
    pub opt_enabled: bool,
    pub opt_verified: Option<bool>,
    #[serde(skip_serializing)]
    pub password: String,
    pub updated_at: DateTime<Utc>,
    pub username: String,
}

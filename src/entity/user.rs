use chrono::{DateTime, Utc};
use scylla::{
    cql_to_rust::{FromCqlVal, FromCqlValError},
    FromRow,
    _macro_internal::CqlValue,
};
use serde::{Deserialize, Serialize};

#[repr(i16)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User = 1,
    Admin = 2,
}

impl FromCqlVal<CqlValue> for Role {
    fn from_cql(cql: scylla::frame::response::result::CqlValue) -> Result<Self, FromCqlValError>
    where
        Self: Sized,
    {
        let i = i16::from_cql(cql)?;
        match i {
            1 => Ok(Role::User),
            2 => Ok(Role::Admin),
            _ => Err(FromCqlValError::BadCqlType),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: Role,
    pub updated_at: DateTime<Utc>,
    pub username: String,
}

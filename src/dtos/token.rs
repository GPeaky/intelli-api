use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};

//* Token Type Enum
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum TokenType {
    Bearer,
    Email,
    ResetPassword,
    RefreshBearer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenClaim {
    pub exp: usize,
    pub sub: i32,
    pub token_type: TokenType,
}

// Token Type Implementation
impl TokenType {
    pub fn set_expiration(&self) -> usize {
        let minutes = match self {
            TokenType::RefreshBearer => Duration::days(7),
            TokenType::Bearer => Duration::days(1),
            _ => Duration::minutes(15),
        };

        Local::now()
            .checked_add_signed(minutes)
            .unwrap()
            .timestamp() as usize
    }

    pub fn base_key(&self) -> &str {
        match self {
            TokenType::Email => "tokens:email",
            TokenType::ResetPassword => "tokens:reset_password",
            TokenType::RefreshBearer => "tokens:refresh_access",
            _ => panic!("Invalid token type"),
        }
    }
}

use chrono::{Duration, Utc};
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
    pub sub: String,
    pub token_type: TokenType,
}

// Token Type Implementation
impl TokenType {
    pub fn get_expiration(&self) -> usize {
        let minutes = match self {
            TokenType::RefreshBearer => Duration::days(7),
            TokenType::Bearer => Duration::days(1),
            _ => Duration::minutes(15),
        };

        Utc::now().checked_add_signed(minutes).unwrap().timestamp() as usize
    }
}

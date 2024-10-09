use tokio::time::Instant;

use chrono::{Duration, Local, TimeDelta};
use serde::{Deserialize, Serialize};

// Token Types and Claims
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum TokenPurpose {
    Authentication,
    EmailVerification,
    PasswordReset,
    RefreshAuthentication,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenPayload {
    pub exp: usize,
    pub subject_id: i32,
    pub purpose: TokenPurpose,
}

// Token Purpose Implementation
impl TokenPurpose {
    pub fn expiry_instant(&self) -> Instant {
        Instant::now() + self.validity_duration().unwrap().to_std().unwrap()
    }

    pub fn expiration_timestamp(&self) -> usize {
        let duration = self.validity_duration().unwrap();

        Local::now()
            .checked_add_signed(duration)
            .unwrap()
            .timestamp() as usize
    }

    const fn validity_duration(&self) -> Option<TimeDelta> {
        match self {
            TokenPurpose::RefreshAuthentication => Duration::try_days(17),
            TokenPurpose::Authentication => Duration::try_days(1),
            _ => Duration::try_minutes(15),
        }
    }
}

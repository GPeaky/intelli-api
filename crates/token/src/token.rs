use std::time::Instant;

use base64_simd::{Out, URL_SAFE_NO_PAD};
use chrono::Duration;
use error::{AppResult, TokenError};
use ring::rand::{SecureRandom, SystemRandom};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token([u8; 16]);

#[derive(PartialEq)]
pub enum TokenIntent {
    Auth,
    RefreshAuth,
    EmailVerify,
    PasswordReset,
}

pub(crate) struct TokenEntry {
    pub(crate) intent: TokenIntent,
    pub(crate) expiry: Instant,
    pub(crate) id: i32,
}

impl TokenIntent {
    pub(crate) const fn lifespan(&self) -> Duration {
        match self {
            TokenIntent::Auth => Duration::days(1),
            TokenIntent::RefreshAuth => Duration::days(7),
            TokenIntent::EmailVerify => Duration::minutes(20),
            TokenIntent::PasswordReset => Duration::minutes(30),
        }
    }
}

impl Token {
    #[inline]
    pub(crate) fn new() -> Self {
        let rng = SystemRandom::new();
        let mut token = [0u8; 16];

        rng.fill(&mut token)
            .expect("Failed generating random bytes");

        Self(token)
    }

    pub fn as_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode_to_string(self.0)
    }

    pub fn from_base64(str: &str) -> AppResult<Token> {
        let mut token = [0u8; 16];

        match URL_SAFE_NO_PAD.decode(str.as_bytes(), Out::from_slice(&mut token)) {
            Ok(_) => Ok(Self(token)),
            _ => Err(TokenError::InvalidToken)?,
        }
    }
}

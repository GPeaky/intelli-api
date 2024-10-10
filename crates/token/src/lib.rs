use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use error::{AppResult, TokenError};
use token::TokenEntry;

mod token;

pub use token::{Token, TokenIntent};
use tokio::{task, time::sleep};

const MAX_TOKENS_PER_USER: usize = 10;
const PURGE_INTERVAL: Duration = Duration::from_secs(3600);

pub struct TokenManager {
    tokens: DashMap<Token, TokenEntry>,
    user_tokens: DashMap<i32, VecDeque<Token>>,
}

impl TokenManager {
    #[inline]
    pub fn new() -> Self {
        Self {
            tokens: DashMap::with_capacity(1000),
            user_tokens: DashMap::with_capacity(100),
        }
    }

    pub fn create(&self, id: i32, intent: TokenIntent) -> Token {
        let mut user_tokens = self.user_tokens.entry(id).or_default();

        if user_tokens.len() >= MAX_TOKENS_PER_USER {
            if let Some(oldest_token) = user_tokens.pop_front() {
                self.tokens.remove(&oldest_token);
            }
        }

        let token = Token::new();
        let entry = TokenEntry {
            id,
            expiry: Instant::now() + intent.lifespan().to_std().unwrap(),
            intent,
        };

        self.tokens.insert(token, entry);
        user_tokens.push_back(token);

        token
    }

    pub fn validate(&self, token: &Token, intent: TokenIntent) -> AppResult<i32> {
        let now = Instant::now();

        if let Some(entry) = self.tokens.get(token) {
            if entry.expiry <= now {
                self.tokens.remove(token);
                return Err(TokenError::ExpiredToken)?;
            }

            if entry.intent != intent {
                return Err(TokenError::InvalidToken)?;
            }

            return Ok(entry.id);
        }

        Err(TokenError::InvalidToken)?
    }

    pub fn remove(&self, token: &Token, intent: TokenIntent) -> bool {
        if let Some((_, entry)) = self.tokens.remove_if(token, |_, e| e.intent == intent) {
            self.user_tokens.entry(entry.id).and_modify(|tokens| {
                tokens.retain(|t| t != token);
            });
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn start_purge_thread(&'static self) {
        tokio::spawn(async move {
            loop {
                sleep(PURGE_INTERVAL).await;
                task::spawn_blocking(move || {
                    self.purge_expired();
                });
            }
        });
    }

    #[inline]
    fn purge_expired(&self) {
        let now = Instant::now();
        let expired_tokens: Vec<_> = self
            .tokens
            .iter()
            .filter(|r| r.expiry <= now)
            .map(|r| (*r.key(), r.id))
            .collect();

        for (token, id) in expired_tokens {
            self.tokens.remove(&token);
            self.user_tokens.entry(id).and_modify(|tokens| {
                tokens.retain(|t| t != &token);
            });
        }
    }
}

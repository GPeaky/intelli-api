use std::{collections::VecDeque, time::Duration};

use dashmap::DashMap;
use error::{AppResult, TokenError};
use token::TokenEntry;

mod persistence;
mod token;

pub use token::{Token, TokenIntent};
use tokio::{task, time::sleep};
use utils::current_timestamp_s;

const MAX_TOKENS_PER_USER: usize = 10;
const PURGE_INTERVAL: Duration = Duration::from_secs(900);

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

    #[inline]
    pub fn load_from_file() -> std::io::Result<Self> {
        persistence::TokenManagerPersistence::load()
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
            expiry: current_timestamp_s() + intent.lifespan(),
            intent,
        };

        self.tokens.insert(token, entry);
        user_tokens.push_back(token);

        token
    }

    pub fn validate(&self, token: &Token, intent: TokenIntent) -> AppResult<i32> {
        let now = current_timestamp_s();

        if let Some(entry) = self.tokens.get(token) {
            if entry.expiry <= now {
                drop(entry);
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
                    self.save_to_file().unwrap();
                });
            }
        });
    }

    #[inline]
    fn save_to_file(&self) -> std::io::Result<()> {
        persistence::TokenManagerPersistence::save(self)
    }

    #[inline]
    fn purge_expired(&self) {
        let now = current_timestamp_s();
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

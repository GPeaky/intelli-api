use std::time::Duration;

use quick_cache::sync::Cache;
use tokio::time::Instant;

use crate::structs::TokenType;

pub struct TokenCache {
    cache: Cache<(String, TokenType), Instant>,
    refresh_tokens: Cache<(i32, String), (Instant, String)>,
}

// Todo - Parse possibles errors
impl TokenCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::new(10_000),
            refresh_tokens: Cache::new(10_000),
        }
    }

    pub fn set_token(&self, token: String, token_type: TokenType) {
        let expiry = token_type.expiry();
        self.cache.insert((token, token_type), expiry)
    }

    pub fn set_refresh_token(&self, user_id: i32, fingerprint: String, token: String) {
        let expiry = Instant::now() + Duration::from_secs(86400);
        self.refresh_tokens
            .insert((user_id, fingerprint), (expiry, token));
    }

    pub fn get_token(&self, token: &str, token_type: TokenType) -> bool {
        if let Some(expiry) = self.cache.get(&(token.to_owned(), token_type.clone())) {
            if Instant::now() < expiry {
                return true;
            } else {
                self.remove_token(token, token_type)
            }
        }

        false
    }

    pub fn get_refresh_token(&self, user_id: i32, fingerprint: &str) -> Option<String> {
        if let Some((expiry, token)) = self.refresh_tokens.get(&(user_id, fingerprint.to_string()))
        {
            if Instant::now() < expiry {
                return Some(token);
            } else {
                self.refresh_tokens
                    .remove(&(user_id, fingerprint.to_string()));
            }
        }

        None
    }

    pub fn remove_token(&self, token: &str, token_type: TokenType) {
        self.cache.remove(&(token.to_owned(), token_type));
    }

    pub fn remove_refresh_token(&self, user_id: i32, fingerprint: &str) {
        self.refresh_tokens
            .remove(&(user_id, fingerprint.to_string()));
    }
}

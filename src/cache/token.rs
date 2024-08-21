use std::time::Duration;

use quick_cache::sync::Cache;
use tokio::time::Instant;

use crate::structs::TokenPurpose;

use super::CACHE_CAPACITY;

// TODO: Make this impl a bit more legible
pub struct TokenCache {
    cache: Cache<(String, TokenPurpose), Instant>,
    refresh_tokens: Cache<(i32, String), (Instant, String)>,
}

// TODO - Parse possibles errors
impl TokenCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::new(CACHE_CAPACITY),
            refresh_tokens: Cache::new(CACHE_CAPACITY),
        }
    }

    pub fn set_token(&self, token: String, token_type: TokenPurpose) {
        let expiry = token_type.expiry_instant();
        self.cache.insert((token, token_type), expiry)
    }

    pub fn set_refresh_token(&self, user_id: i32, fingerprint: String, token: String) {
        let expiry = Instant::now() + Duration::from_secs(86400);

        self.refresh_tokens
            .insert((user_id, fingerprint), (expiry, token));
    }

    pub fn get_token(&self, token: String, token_type: TokenPurpose) -> bool {
        if let Some(expiry) = self.cache.get(&(token.clone(), token_type)) {
            if Instant::now() < expiry {
                return true;
            } else {
                self.remove_token(token, token_type)
            }
        }

        false
    }

    pub fn get_refresh_token(&self, user_id: i32, fingerprint: String) -> Option<String> {
        if let Some((expiry, token)) = self.refresh_tokens.get(&(user_id, fingerprint.clone())) {
            if Instant::now() < expiry {
                return Some(token);
            } else {
                self.refresh_tokens.remove(&(user_id, fingerprint));
            }
        }

        None
    }

    pub fn remove_token(&self, token: String, token_type: TokenPurpose) {
        self.cache.remove(&(token, token_type));
    }

    pub fn remove_refresh_token(&self, user_id: i32, fingerprint: String) {
        self.refresh_tokens.remove(&(user_id, fingerprint));
    }
}

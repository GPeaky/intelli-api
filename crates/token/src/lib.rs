use std::time::Instant;

use dashmap::DashMap;
use token::{Token, TokenEntry, TokenIntent};

mod token;

pub struct TokenManager {
    tokens: DashMap<Token, TokenEntry>,
}

impl TokenManager {
    #[inline]
    // Spwn thread for purging expired tokens
    pub fn new() -> Self {
        Self {
            tokens: DashMap::new(),
        }
    }

    pub fn create(&self, id: i32, intent: TokenIntent) -> Token {
        let token = Token::new();
        let entry = TokenEntry {
            id,
            expiry: Instant::now() + intent.lifespan().to_std().unwrap(),
            intent,
        };

        self.tokens.insert(token, entry);
        token
    }

    pub fn validate(&self, token: &Token, intent: TokenIntent) -> Option<i32> {
        let now = Instant::now();

        if let Some(entry) = self.tokens.get(token) {
            if entry.expiry <= now || entry.intent != intent {
                self.tokens.remove(token);
                return None;
            }
            return Some(entry.id);
        }

        None
    }
    
    fn purge_expired(&self) {
        let now = Instant::now();
        self.tokens.retain(|_, entry| entry.expiry > now);
    }
}

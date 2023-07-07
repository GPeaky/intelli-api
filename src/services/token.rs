use std::sync::Arc;

use crate::{
    config::Database,
    dtos::{TokenClaim, TokenType},
    error::{AppResult, TokenError},
};
use axum::async_trait;
use dotenvy::var;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use redis::AsyncCommands;

#[derive(Clone)]
pub struct TokenService {
    header: Header,
    validation: Validation,
    db_conn: Arc<Database>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[async_trait]
pub trait TokenServiceTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>>;
    fn generate_token(&self, sub: &str, token_type: TokenType) -> AppResult<String>;
    async fn generate_refresh_token(&self, user_id: String, device_id: &str) -> AppResult<String>;
    async fn remove_refresh_token(&self, user_id: String, device_id: &str) -> AppResult<()>;
}

#[async_trait]
impl TokenServiceTrait for TokenService {
    fn new(db_conn: &Arc<Database>) -> Self {
        let secret = var("JWT_SECRET").unwrap();

        Self {
            header: Header::default(),
            validation: Validation::default(),
            db_conn: Arc::clone(db_conn),
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    fn generate_token(&self, sub: &str, token_type: TokenType) -> AppResult<String> {
        let token_claim = TokenClaim {
            exp: token_type.get_expiration(),
            sub: sub.to_owned(),
            token_type,
        };

        encode(&self.header, &token_claim, &self.encoding_key)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
    }

    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>> {
        decode::<TokenClaim>(token, &self.decoding_key, &self.validation)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
    }

    async fn generate_refresh_token(&self, user_id: String, device_id: &str) -> AppResult<String> {
        let token = self.generate_token(device_id, TokenType::RefreshBearer)?;

        self.db_conn
            .get_redis()
            .await
            .set_ex(
                format!("rf_tokens:{}:{}", user_id, device_id),
                &token,
                7 * 24 * 60 * 60,
            )
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(token)
    }

    async fn remove_refresh_token(&self, user_id: String, device_id: &str) -> AppResult<()> {
        self.db_conn
            .get_redis()
            .await
            .del(format!("rf_tokens:{}:{}", user_id, device_id))
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(())
    }
}

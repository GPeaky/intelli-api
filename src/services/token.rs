use std::sync::Arc;

use crate::{
    config::Database,
    dtos::{TokenClaim, TokenType},
    error::{AppResult, TokenError},
};
use axum::async_trait;
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
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String>;
    async fn generate_refresh_token(&self, user_id: String, fingerprint: &str)
        -> AppResult<String>;
    async fn remove_refresh_token(&self, user_id: String, fingerprint: &str) -> AppResult<()>;
}

#[async_trait]
impl TokenServiceTrait for TokenService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            header: Header::new(jsonwebtoken::Algorithm::RS256),
            validation: Validation::new(jsonwebtoken::Algorithm::RS256),
            db_conn: Arc::clone(db_conn),
            encoding_key: EncodingKey::from_rsa_pem(include_bytes!("../../certs/jsonwebtoken.key"))
                .unwrap(),
            decoding_key: DecodingKey::from_rsa_pem(include_bytes!("../../certs/jsonwebtoken.crt"))
                .unwrap(),
        }
    }

    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>> {
        decode::<TokenClaim>(token, &self.decoding_key, &self.validation)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
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

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String> {
        let token = self.validate(refresh_token)?;

        if token.claims.token_type != TokenType::RefreshBearer {
            Err(TokenError::InvalidTokenType)?
        }

        let db_token: String = self
            .db_conn
            .get_redis_async()
            .await
            .get(format!("rf_tokens:{}:{}", token.claims.sub, fingerprint))
            .await
            .map_err(|_| TokenError::TokenExpired)?;

        if db_token != refresh_token {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(&token.claims.sub, TokenType::Bearer)
    }

    async fn generate_refresh_token(
        &self,
        user_id: String,
        fingerprint: &str,
    ) -> AppResult<String> {
        let token = self.generate_token(&user_id, TokenType::RefreshBearer)?;

        self.db_conn
            .get_redis_async()
            .await
            .set_ex(
                format!("rf_tokens:{}:{}", user_id, fingerprint),
                &token,
                7 * 24 * 60 * 60,
            )
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(token)
    }

    async fn remove_refresh_token(&self, user_id: String, fingerprint: &str) -> AppResult<()> {
        self.db_conn
            .get_redis_async()
            .await
            .del(format!("rf_tokens:{}:{}", user_id, fingerprint))
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(())
    }
}

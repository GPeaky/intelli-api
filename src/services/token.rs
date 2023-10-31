use crate::{
    config::Database,
    dtos::{TokenClaim, TokenType},
    error::{AppResult, TokenError},
};
use axum::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use redis::AsyncCommands;
use std::{fs, sync::Arc};

const AUTH_TOKEN_EXPIRATION: usize = 15 * 60;
const REFRESH_TOKEN_EXPIRATION: usize = 7 * 24 * 60 * 60;

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
    async fn save_reset_password_token(&self, token: &str) -> AppResult<()>;
    async fn save_email_token(&self, token: &str) -> AppResult<()>;
    async fn generate_token(&self, sub: u32, token_type: TokenType) -> AppResult<String>;
    async fn remove_refresh_token(&self, user_id: &u32, fingerprint: &str) -> AppResult<()>;
    async fn generate_refresh_token(&self, user_id: &u32, fingerprint: &str) -> AppResult<String>;
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String>;
}

#[async_trait]
impl TokenServiceTrait for TokenService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            header: Header::new(jsonwebtoken::Algorithm::RS256),
            validation: Validation::new(jsonwebtoken::Algorithm::RS256),
            db_conn: Arc::clone(db_conn),
            encoding_key: EncodingKey::from_rsa_pem(
                &fs::read("certs/jsonwebtoken.key").expect("Unable to read key"),
            )
            .unwrap(),
            decoding_key: DecodingKey::from_rsa_pem(
                &fs::read("certs/jsonwebtoken.crt").expect("Unable to read key"),
            )
            .unwrap(),
        }
    }

    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>> {
        decode::<TokenClaim>(token, &self.decoding_key, &self.validation)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
    }

    async fn generate_token(&self, sub: u32, token_type: TokenType) -> AppResult<String> {
        let token_claim = TokenClaim {
            sub,
            exp: token_type.get_expiration(),
            token_type,
        };

        encode(&self.header, &token_claim, &self.encoding_key)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
    }

    async fn save_reset_password_token(&self, token: &str) -> AppResult<()> {
        let mut redis = self.db_conn.get_redis_async().await;

        redis
            .set_ex::<_, u8, ()>(format!("reset:{}", token), 1, AUTH_TOKEN_EXPIRATION)
            .await
            .unwrap();

        Ok(())
    }

    async fn save_email_token(&self, token: &str) -> AppResult<()> {
        let mut redis = self.db_conn.get_redis_async().await;

        redis
            .set_ex::<_, u8, ()>(format!("email:{}", token), 1, AUTH_TOKEN_EXPIRATION)
            .await
            .unwrap();

        Ok(())
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String> {
        let token = self.validate(refresh_token)?;

        if token.claims.token_type.ne(&TokenType::RefreshBearer) {
            Err(TokenError::InvalidTokenType)?
        }

        let db_token: String = self
            .db_conn
            .get_redis_async()
            .await
            .get(format!("rf_tokens:{}:{}", token.claims.sub, fingerprint))
            .await
            .map_err(|_| TokenError::TokenNotFound)?;

        if db_token.ne(refresh_token) {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(token.claims.sub, TokenType::Bearer)
            .await
    }

    async fn generate_refresh_token(&self, user_id: &u32, fingerprint: &str) -> AppResult<String> {
        let token = self
            .generate_token(*user_id, TokenType::RefreshBearer)
            .await?;

        self.db_conn
            .get_redis_async()
            .await
            .set_ex(
                format!("rf_tokens:{}:{}", user_id, fingerprint),
                &token,
                REFRESH_TOKEN_EXPIRATION,
            )
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(token)
    }

    async fn remove_refresh_token(&self, user_id: &u32, fingerprint: &str) -> AppResult<()> {
        self.db_conn
            .get_redis_async()
            .await
            .del(format!("rf_tokens:{}:{}", user_id, fingerprint))
            .await
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(())
    }
}

use crate::{
    cache::RedisCache,
    dtos::{TokenClaim, TokenType},
    error::{AppResult, TokenError},
};
use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use ntex::rt::spawn_blocking;
use std::{fs, sync::Arc};

pub struct TokenService {
    header: Arc<Header>,
    cache: Arc<RedisCache>,
    validation: Validation,
    encoding_key: Arc<EncodingKey>,
    decoding_key: DecodingKey,
}

#[async_trait]
pub trait TokenServiceTrait {
    fn new(cache: &Arc<RedisCache>) -> Self;
    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>>;
    async fn save_reset_password_token(&self, token: &str) -> AppResult<()>;
    async fn save_email_token(&self, token: &str) -> AppResult<()>;
    async fn generate_token(&self, sub: i32, token_type: TokenType) -> AppResult<String>;
    async fn remove_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<()>;
    async fn generate_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<String>;
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String>;
}

#[async_trait]
impl TokenServiceTrait for TokenService {
    fn new(cache: &Arc<RedisCache>) -> Self {
        let header = Header::new(jsonwebtoken::Algorithm::RS256);

        let encoding_key = EncodingKey::from_rsa_pem(
            &fs::read("certs/jsonwebtoken.key").expect("Unable to read key"),
        )
        .unwrap();

        Self {
            cache: cache.clone(),
            header: Arc::from(header),
            encoding_key: Arc::from(encoding_key),
            validation: Validation::new(jsonwebtoken::Algorithm::RS256),
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

    async fn generate_token(&self, sub: i32, token_type: TokenType) -> AppResult<String> {
        let header = self.header.clone();
        let encoding_key = self.encoding_key.clone();
        let token_claim = TokenClaim {
            sub,
            exp: token_type.set_expiration(),
            token_type,
        };

        spawn_blocking(move || {
            encode(&header, &token_claim, &encoding_key)
                .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
        })
        .await
        .unwrap()
    }

    async fn save_reset_password_token(&self, token: &str) -> AppResult<()> {
        self.cache
            .token
            .set_token(token, &TokenType::ResetPassword)
            .await
    }

    async fn save_email_token(&self, token: &str) -> AppResult<()> {
        self.cache.token.set_token(token, &TokenType::Email).await
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String> {
        let id;

        {
            let token = self.validate(refresh_token)?;

            if token.claims.token_type.ne(&TokenType::RefreshBearer) {
                Err(TokenError::InvalidTokenType)?
            }

            id = token.claims.sub;
        }

        let db_token = self.cache.token.get_refresh_token(&id, fingerprint).await?;

        if db_token.ne(refresh_token) {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(id, TokenType::Bearer).await
    }

    async fn generate_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<String> {
        let token = self
            .generate_token(*user_id, TokenType::RefreshBearer)
            .await?;

        self.cache
            .token
            .set_refresh_token(&token, fingerprint)
            .await?;

        Ok(token)
    }

    async fn remove_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<()> {
        self.cache
            .token
            .remove_refresh_token(user_id, fingerprint)
            .await
    }
}

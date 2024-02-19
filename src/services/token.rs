use std::fs;

use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::{
    cache::RedisCache,
    error::{AppResult, TokenError},
    structs::{TokenClaim, TokenType},
};

/// Manages token generation, validation, and lifecycle for user authentication and authorization.
///
/// This service provides functionality for generating, validating, and managing various types of
/// tokens, such as refresh tokens and email verification tokens. It utilizes cryptographic keys
/// for token encoding and decoding, ensuring secure token management.
#[derive(Clone)]
pub struct TokenService {
    /// JWT header configuration.
    header: Header,
    /// Redis cache for storing and retrieving tokens.
    cache: &'static RedisCache,
    /// Token validation configurations.
    validation: Validation,
    /// Encoding key for generating tokens.
    encoding_key: EncodingKey,
    /// Decoding key for validating tokens.
    decoding_key: DecodingKey,
}

// TODO: Remove this trait and implement the methods directly on the `TokenService` struct.
#[async_trait]
pub trait TokenServiceTrait {
    /// Constructs a new `TokenService` instance.
    ///
    /// # Arguments
    /// - `cache`: A reference to the Redis cache for token storage.
    ///
    /// # Returns
    /// A new `TokenService` instance.
    fn new(cache: &'static RedisCache) -> Self;

    /// Validates a token and returns the associated claims.
    ///
    /// # Arguments
    /// - `token`: The token string to validate.
    ///
    /// # Returns
    /// The token data including the claims if the token is valid.
    fn validate(&self, token: &str) -> AppResult<TokenData<TokenClaim>>;

    /// Saves a reset password token to the cache.
    ///
    /// # Arguments
    /// - `token`: The reset password token to save.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn save_reset_password_token(&self, token: &str) -> AppResult<()>;

    /// Saves an email verification token to the cache.
    ///
    /// # Arguments
    /// - `token`: The email verification token to save.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn save_email_token(&self, token: &str) -> AppResult<()>;

    /// Generates a new token with specified subject and type.
    ///
    /// # Arguments
    /// - `sub`: The subject ID the token is issued for.
    /// - `token_type`: The type of the token being generated.
    ///
    /// # Returns
    /// A new token as a string if successful.
    async fn generate_token(&self, sub: i32, token_type: TokenType) -> AppResult<String>;

    /// Removes a refresh token from the cache.
    ///
    /// # Arguments
    /// - `user_id`: The ID of the user the token belongs to.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    async fn remove_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<()>;

    /// Generates a new refresh token for a user.
    ///
    /// # Arguments
    /// - `user_id`: The ID of the user to generate the token for.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// A new refresh token as a string if successful.
    async fn generate_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<String>;

    /// Refreshes an access token using a refresh token.
    ///
    /// # Arguments
    /// - `refresh_token`: The refresh token to validate and use for generating a new access token.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// A new access token as a string if successful.
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String>;
}

#[async_trait]
impl TokenServiceTrait for TokenService {
    fn new(cache: &'static RedisCache) -> Self {
        Self {
            cache,
            header: Header::new(jsonwebtoken::Algorithm::RS256),
            encoding_key: EncodingKey::from_rsa_pem(
                &fs::read("certs/jsonwebtoken.key").expect("Unable to read key"),
            )
            .unwrap(),
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

    async fn save_reset_password_token(&self, token: &str) -> AppResult<()> {
        self.cache
            .token
            .set_token(token, &TokenType::ResetPassword)
            .await
    }

    async fn save_email_token(&self, token: &str) -> AppResult<()> {
        self.cache.token.set_token(token, &TokenType::Email).await
    }

    async fn generate_token(&self, sub: i32, token_type: TokenType) -> AppResult<String> {
        let token_claim = TokenClaim {
            sub,
            exp: token_type.set_expiration(),
            token_type,
        };

        encode(&self.header, &token_claim, &self.encoding_key)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()).into())
    }

    async fn remove_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<()> {
        self.cache
            .token
            .remove_refresh_token(user_id, fingerprint)
            .await
    }

    async fn generate_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<String> {
        let token = self
            .generate_token(user_id, TokenType::RefreshBearer)
            .await?;

        self.cache
            .token
            .set_refresh_token(&token, fingerprint)
            .await?;

        Ok(token)
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: &str,
    ) -> AppResult<String> {
        let id = {
            let token = self.validate(refresh_token)?;

            if token.claims.token_type != TokenType::RefreshBearer {
                Err(TokenError::InvalidTokenType)?
            }

            token.claims.sub
        };

        let db_token = self.cache.token.get_refresh_token(id, fingerprint).await?;

        if db_token != refresh_token {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(id, TokenType::Bearer).await
    }
}

use deadpool_redis::redis::AsyncCommands;

use crate::error::CommonError;
use crate::{
    config::{constants::*, Database},
    error::AppResult,
    structs::TokenType,
};

/// `TokenCache` is a caching structure for storing and retrieving authentication tokens using Redis.
/// It provides methods to interact with a Redis cache to set, get, and remove tokens.
///
#[derive(Clone)]
pub struct TokenCache {
    db: Database,
}

impl TokenCache {
    /// Creates a new `TokenCache` instance with the provided `Database` reference.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database used for caching.
    ///
    /// # Returns
    ///
    /// A new `TokenCache` instance.
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    /// Sets a token in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token to be stored.
    /// * `token_type` - The type of the token (e.g., Bearer or Email).
    ///
    /// # Errors
    ///
    /// If the `token_type` is `RefreshBearer`, an `InvalidUsedFeature` error is returned
    /// as refresh tokens cannot be set.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token is successfully set in the cache.
    ///
    pub async fn set_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        if token_type == &TokenType::RefreshBearer {
            Err(CommonError::InvalidUsedFeature(
                "Refresh token can't be set".to_string(),
            ))?;
        }

        let mut conn = self.db.redis.get().await?;

        conn.set_ex(
            &format!("{}:{token}", token_type.base_key()),
            1,
            GENERIC_TOKEN_EXPIRATION,
        )
        .await?;

        Ok(())
    }

    /// Retrieves a token from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token to retrieve.
    /// * `token_type` - The type of the token (e.g., Bearer or Email).
    ///
    /// # Errors
    ///
    /// If the `token_type` is `RefreshBearer`, an `InvalidUsedFeature` error is returned
    /// as refresh tokens cannot be set.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token is successfully retrieved from the cache.
    ///
    pub async fn get_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        if token_type == &TokenType::RefreshBearer {
            Err(CommonError::InvalidUsedFeature(
                "Refresh token can't be set".to_string(),
            ))?;
        }

        let mut conn = self.db.redis.get().await?;

        conn.get(&format!("{}:{}", token_type.base_key(), token))
            .await?;

        Ok(())
    }

    /// Removes an authentication token from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token to remove.
    /// * `token_type` - The type of the token (e.g., AccessToken or RefreshToken).
    ///
    /// # Errors
    ///
    /// If the `token_type` is `RefreshBearer`, an `InvalidUsedFeature` error is returned
    /// as refresh tokens cannot be set.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token is successfully removed from the cache.
    ///
    pub async fn remove_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        if token_type == &TokenType::RefreshBearer {
            Err(CommonError::InvalidUsedFeature(
                "Refresh token can't be set".to_string(),
            ))?;
        }

        let mut conn = self.db.redis.get().await?;

        conn.del(&format!("{}:{}", token_type.base_key(), token))
            .await?;

        Ok(())
    }

    /// Sets a refresh token in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to be stored.
    /// * `fingerprint` - The fingerprint associated with the refresh token.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the refresh token is successfully set in the cache.
    ///
    pub async fn set_refresh_token(&self, token: &str, fingerprint: &str) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.set_ex(
            &format!("{}:{}", TokenType::RefreshBearer.base_key(), fingerprint),
            token,
            REFRESH_TOKEN_EXPIRATION,
        )
        .await?;

        Ok(())
    }

    /// Retrieves a refresh token from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID associated with the refresh token.
    /// * `fingerprint` - The fingerprint associated with the refresh token.
    ///
    /// # Returns
    ///
    /// Returns the retrieved refresh token as a `String` or an error if not found.
    ///
    pub async fn get_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<String> {
        let mut conn = self.db.redis.get().await?;

        let token = conn
            .get(&format!(
                "{}:{}:{}",
                TokenType::RefreshBearer.base_key(),
                user_id,
                fingerprint
            ))
            .await?;

        Ok(token)
    }

    /// Removes a refresh token from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID associated with the refresh token.
    /// * `fingerprint` - The fingerprint associated with the refresh token.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the refresh token is successfully removed from the cache.
    ///
    pub async fn remove_refresh_token(&self, user_id: i32, fingerprint: &str) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del(&format!(
            "{}:{}:{}",
            TokenType::RefreshBearer.base_key(),
            user_id,
            fingerprint
        ))
        .await?;

        Ok(())
    }
}

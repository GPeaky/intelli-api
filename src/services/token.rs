use std::fs;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::{
    cache::ServiceCache,
    error::{AppResult, TokenError},
    structs::{TokenPayload, TokenPurpose},
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
    cache: &'static ServiceCache,
    /// Token validation configurations.
    validation: Validation,
    /// Encoding key for generating tokens.
    encoding_key: EncodingKey,
    /// Decoding key for validating tokens.
    decoding_key: DecodingKey,
}

impl TokenService {
    /// Constructs a new `TokenService` instance.
    ///
    /// # Arguments
    /// - `cache`: A reference to the Redis cache for token storage.
    ///
    /// # Returns
    /// A new `TokenService` instance.
    pub fn new(cache: &'static ServiceCache) -> Self {
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

    /// Validates a token and returns the associated claims.
    ///
    /// # Arguments
    /// - `token`: The token string to validate.
    ///
    /// # Returns
    /// The token data including the claims if the token is valid.
    pub fn validate(&self, token: &str) -> AppResult<TokenData<TokenPayload>> {
        decode::<TokenPayload>(token, &self.decoding_key, &self.validation)
            .map_err(|_| TokenError::InvalidToken.into())
    }

    /// Saves a reset password token to the cache.
    ///
    /// # Arguments
    /// - `token`: The reset password token to save.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    pub fn save_reset_password_token(&self, token: String) {
        self.cache
            .token
            .set_token(token, TokenPurpose::PasswordReset);
    }

    /// Saves an email verification token to the cache.
    ///
    /// # Arguments
    /// - `token`: The email verification token to save.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    #[inline]
    pub fn save_email_token(&self, token: String) {
        self.cache
            .token
            .set_token(token, TokenPurpose::EmailVerification);
    }

    /// Generates a new token with specified subject and type.
    ///
    /// # Arguments
    /// - `sub`: The subject ID the token is issued for.
    /// - `token_type`: The type of the token being generated.
    ///
    /// # Returns
    /// A new token as a string if successful.
    pub fn generate_token(&self, subject_id: i32, purpose: TokenPurpose) -> AppResult<String> {
        let token_claim = TokenPayload {
            subject_id,
            expiration: purpose.expiration_timestamp(),
            purpose,
        };

        encode(&self.header, &token_claim, &self.encoding_key)
            .map_err(|_| TokenError::TokenCreationError.into())
    }

    /// Removes a refresh token from the cache.
    ///
    /// # Arguments
    /// - `user_id`: The ID of the user the token belongs to.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// An empty result indicating success or failure.
    pub fn remove_refresh_token(&self, user_id: i32, fingerprint: String) {
        self.cache.token.remove_refresh_token(user_id, fingerprint);
    }

    /// Generates a new refresh token for a user.
    ///
    /// # Arguments
    /// - `user_id`: The ID of the user to generate the token for.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// A new refresh token as a string if successful.
    pub fn generate_refresh_token(&self, user_id: i32, fingerprint: String) -> AppResult<String> {
        let token = self.generate_token(user_id, TokenPurpose::RefreshAuthentication)?;

        self.cache
            .token
            .set_refresh_token(user_id, fingerprint, token.clone());

        Ok(token)
    }

    /// Refreshes an access token using a refresh token.
    ///
    /// # Arguments
    /// - `refresh_token`: The refresh token to validate and use for generating a new access token.
    /// - `fingerprint`: A unique identifier for the user's device.
    ///
    /// # Returns
    /// A new access token as a string if successful.
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        fingerprint: String,
    ) -> AppResult<String> {
        let id = {
            let token = self.validate(refresh_token)?;

            if token.claims.purpose != TokenPurpose::RefreshAuthentication {
                Err(TokenError::InvalidTokenPurpose)?
            }

            token.claims.subject_id
        };

        let Some(db_token) = self.cache.token.get_refresh_token(id, fingerprint) else {
            Err(TokenError::MissingToken)?
        };

        if db_token != refresh_token {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(id, TokenPurpose::Authentication)
    }
}

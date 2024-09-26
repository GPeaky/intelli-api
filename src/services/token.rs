use crate::{
    config::Database,
    error::{AppResult, TokenError},
    structs::{TokenPayload, TokenPurpose},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use std::fs;

// TODO: Update this implementation

/// Manages token lifecycle for authentication and authorization.
#[derive(Clone)]
pub struct TokenService {
    header: Header,
    validation: Validation,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    db: &'static Database,
}

impl TokenService {
    /// Creates a new `TokenService` instance.
    pub fn new(db: &'static Database) -> Self {
        Self {
            db,
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

    #[inline]
    pub fn subject_id(&self, token: &str) -> AppResult<i32> {
        let token_data = self.validate(token)?;
        Ok(token_data.claims.subject_id)
    }

    /// Validates a token and returns the associated claims.
    pub fn validate(&self, token: &str) -> AppResult<TokenData<TokenPayload>> {
        decode::<TokenPayload>(token, &self.decoding_key, &self.validation)
            .map_err(|_| TokenError::InvalidToken.into())
    }

    /// Saves a reset password token to the cache.
    pub fn save_reset_password_token(&self, token: String) {
        self.db
            .cache
            .token
            .set_token(token, TokenPurpose::PasswordReset);
    }

    /// Saves an email verification token to the cache.
    #[inline]
    pub fn save_email_token(&self, token: String) {
        self.db
            .cache
            .token
            .set_token(token, TokenPurpose::EmailVerification);
    }

    /// Generates a new token with specified subject and purpose.
    pub fn generate_token(&self, subject_id: i32, purpose: TokenPurpose) -> AppResult<String> {
        let token_claim = TokenPayload {
            subject_id,
            exp: purpose.expiration_timestamp(),
            purpose,
        };
        encode(&self.header, &token_claim, &self.encoding_key)
            .map_err(|_| TokenError::TokenCreationError.into())
    }

    /// Removes a refresh token from the cache.
    pub fn remove_refresh_token(&self, user_id: i32, fingerprint: String) {
        self.db
            .cache
            .token
            .remove_refresh_token(user_id, fingerprint);
    }

    /// Generates a new refresh token for a user.
    pub fn generate_refresh_token(&self, user_id: i32, fingerprint: String) -> AppResult<String> {
        let token = self.generate_token(user_id, TokenPurpose::RefreshAuthentication)?;
        self.db
            .cache
            .token
            .set_refresh_token(user_id, fingerprint, token.clone());
        Ok(token)
    }

    /// Refreshes an access token using a refresh token.
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

        let Some(db_token) = self.db.cache.token.get_refresh_token(id, fingerprint) else {
            Err(TokenError::MissingToken)?
        };

        if db_token != refresh_token {
            Err(TokenError::InvalidToken)?
        }

        self.generate_token(id, TokenPurpose::Authentication)
    }
}

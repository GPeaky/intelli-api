use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use tokio::sync::Semaphore;

use crate::{
    config::constants::{PASS_ALG, PASS_CREDENTIAL_LEN, PASS_ITERATIONS, PASS_SALT_LEN},
    error::{AppResult, CommonError},
};

pub struct PasswordHasher {
    semaphore: Semaphore,
}

impl PasswordHasher {
    pub fn new(max_concurrents: usize) -> PasswordHasher {
        PasswordHasher {
            semaphore: Semaphore::new(max_concurrents),
        }
    }

    pub async fn hash_password(&self, password: String) -> AppResult<String> {
        let _permit = self.semaphore.acquire().await.unwrap();

        ntex::rt::spawn_blocking(move || {
            let mut salt = [0u8; PASS_SALT_LEN];
            let rng = SystemRandom::new();

            rng.fill(&mut salt)
                .map_err(|_| CommonError::HashingFailed)?;

            let mut hash = [0u8; PASS_CREDENTIAL_LEN];
            pbkdf2::derive(
                PASS_ALG,
                PASS_ITERATIONS,
                &salt,
                password.as_bytes(),
                &mut hash,
            );

            let mut hashed_password = [0u8; PASS_SALT_LEN + PASS_CREDENTIAL_LEN];
            hashed_password[..PASS_SALT_LEN].copy_from_slice(&salt);
            hashed_password[PASS_SALT_LEN..].copy_from_slice(&hash);

            Ok(STANDARD.encode(hashed_password))
        })
        .await
        .unwrap_or_else(|_| Err(CommonError::InternalServerError)?)
    }

    pub async fn verify_password(&self, encoded: String, password: String) -> AppResult<bool> {
        let _permit = self.semaphore.acquire().await.unwrap();

        ntex::rt::spawn_blocking(move || {
            let mut combined = [0u8; PASS_SALT_LEN + PASS_CREDENTIAL_LEN];

            STANDARD
                .decode_slice_unchecked(encoded, &mut combined) // Unchecked because the size of the hash is static
                .map_err(|_| CommonError::HashingFailed)?;

            let (salt, hash) = combined.split_at(PASS_SALT_LEN);

            Ok(pbkdf2::verify(PASS_ALG, PASS_ITERATIONS, salt, password.as_bytes(), hash).is_ok())
        })
        .await
        .unwrap_or_else(|_| Err(CommonError::InternalServerError)?)
    }
}

use std::num::NonZeroU32;

use base64_simd::{Out, STANDARD};
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use tokio::sync::Semaphore;

const PASS_SALT_LEN: usize = 16;
const PASS_CREDENTIAL_LEN: usize = 32;
const PASS_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };
static PASS_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;

use error::{AppResult, CommonError};

pub struct PasswordHasher {
    semaphore: Semaphore,
    rng: SystemRandom,
}

impl PasswordHasher {
    pub fn new(max_concurrent: usize) -> PasswordHasher {
        PasswordHasher {
            semaphore: Semaphore::new(max_concurrent),
            rng: SystemRandom::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn hash_password(&self, password: String) -> AppResult<String> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let rng = self.rng.clone();

        tokio::task::spawn_blocking(move || {
            let mut salt = [0u8; PASS_SALT_LEN];

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

            Ok(STANDARD.encode_to_string(hashed_password))
        })
        .await
        .unwrap_or_else(|_| Err(CommonError::InternalServerError)?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn verify_password(&self, encoded: Box<str>, password: String) -> AppResult<bool> {
        let _permit = self.semaphore.acquire().await.unwrap();

        tokio::task::spawn_blocking(move || {
            let mut combined = [0u8; PASS_SALT_LEN + PASS_CREDENTIAL_LEN];

            STANDARD
                .decode(encoded.as_bytes(), Out::from_slice(&mut combined)) // Unchecked because the size of the hash is static
                .map_err(|_| CommonError::HashingFailed)?;

            let (salt, hash) = unsafe { combined.split_at_unchecked(PASS_SALT_LEN) };

            Ok(pbkdf2::verify(PASS_ALG, PASS_ITERATIONS, salt, password.as_bytes(), hash).is_ok())
        })
        .await
        .unwrap_or_else(|_| Err(CommonError::InternalServerError)?)
    }
}

#[cfg(test)]
mod tests {
    use error::AppError;

    use super::*;
    use std::sync::Arc;
    use tokio::task;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn hash_and_verify_password() {
        let hasher = Arc::new(PasswordHasher::new(1));
        let password = "my_secure_password".to_string();

        let hashed = hasher.hash_password(password.clone()).await.unwrap();
        assert!(hasher
            .verify_password(hashed.clone().into(), password.clone())
            .await
            .unwrap());

        assert!(!hasher
            .verify_password(hashed.into(), "wrong_password".to_string())
            .await
            .unwrap());
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]

    async fn concurrent_hashing() {
        let hasher = Arc::new(PasswordHasher::new(5));
        let passwords: Vec<String> = (0..20).map(|i| format!("password_{}", i)).collect();

        let mut handles = Vec::new();
        for password in passwords {
            let hasher = Arc::clone(&hasher);
            handles.push(task::spawn(
                async move { hasher.hash_password(password).await },
            ));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        assert_eq!(results.len(), 20);
        assert!(results.iter().all(|r| r.is_ok()));

        let hashes: Vec<_> = results.into_iter().map(|r| r.unwrap()).collect();
        let unique_hashes: std::collections::HashSet<_> = hashes.into_iter().collect();
        assert_eq!(unique_hashes.len(), 20);
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn semaphore_limit() {
        let max_concurrent = 2;
        let hasher = Arc::new(PasswordHasher::new(max_concurrent));
        let passwords: Vec<String> = (0..10).map(|i| format!("password_{}", i)).collect();

        let start = std::time::Instant::now();

        let mut handles = Vec::new();
        for password in passwords {
            let hasher = Arc::clone(&hasher);
            handles.push(task::spawn(
                async move { hasher.hash_password(password).await },
            ));
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        assert!(
            duration.as_millis() > 250,
            "Hashing completed too quickly: {:?}",
            duration
        );
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn timeout_handling() {
        let hasher = Arc::new(PasswordHasher::new(1));
        let password = "timeout_test_password".to_string();

        let slow_hash = timeout(Duration::from_secs(1), hasher.hash_password(password)).await;

        assert!(slow_hash.is_ok(), "Hash operation timed out unexpectedly");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn invalid_encoded_password() {
        let hasher = Arc::new(PasswordHasher::new(1));
        let result = hasher
            .verify_password(
                "invalid_encoded_string".to_string().into(),
                "password".to_string(),
            )
            .await;
        assert!(matches!(
            result,
            Err(AppError::Common(CommonError::HashingFailed))
        ));
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn password_strength() {
        let hasher = Arc::new(PasswordHasher::new(1));
        let weak_password = "123".to_string();
        let strong_password = "veryStrongP@ssw0rd!".to_string();

        let weak_hash = hasher.hash_password(weak_password.clone()).await.unwrap();
        let strong_hash = hasher.hash_password(strong_password.clone()).await.unwrap();

        assert!(hasher
            .verify_password(weak_hash.clone().into(), weak_password)
            .await
            .unwrap());
        assert!(hasher
            .verify_password(strong_hash.clone().into(), strong_password)
            .await
            .unwrap());

        assert_ne!(weak_hash, strong_hash);

        let start = std::time::Instant::now();
        hasher
            .hash_password("benchmark_password".to_string())
            .await
            .unwrap();
        let duration = start.elapsed();
        assert!(
            duration.as_millis() > 10,
            "Hashing might be too fast for security"
        );
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]

    async fn error_propagation() {
        let hasher = Arc::new(PasswordHasher::new(1));

        let result = hasher.hash_password("".to_string()).await;
        assert!(
            result.is_ok(),
            "Empty password should be handled gracefully"
        );

        // Test con una contraseña muy larga
        let long_password = "a".repeat(10000);
        let result = hasher.hash_password(long_password).await;
        assert!(result.is_ok(), "Long password should be handled gracefully");
    }
}

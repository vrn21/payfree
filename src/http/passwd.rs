use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use tokio::task;

pub async fn hash(password: String) -> anyhow::Result<String> {
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow!(e).context("failed to hash password"))
    })
    .await
    .context("panic in hash()")?
}

pub async fn verify(password: String, hash: String) -> anyhow::Result<bool> {
    task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash)
            .map_err(|e| anyhow!(e).context("BUG: password hash invalid"))?;

        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(anyhow!(e).context("failed to verify password")),
        }
    })
    .await
    .context("panic in verify()")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_and_verify_success() {
        let password = "test_password".to_string();
        let hash = hash(password.clone()).await.expect("Hashing failed");
        let is_valid = verify(password, hash).await.expect("Verification failed");
        assert!(is_valid, "Password should verify successfully");
    }

    #[tokio::test]
    async fn test_verify_failure() {
        let password = "test_password".to_string();
        let wrong_password = "wrong_password".to_string();
        let hash = hash(password).await.expect("Hashing failed");
        let is_valid = verify(wrong_password, hash).await.expect("Verification failed");
        assert!(!is_valid, "Wrong password should not verify");
    }
}

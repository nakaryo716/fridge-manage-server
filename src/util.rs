use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{prelude::BASE64_STANDARD, Engine};
use password_hash::{Salt, SaltString};
use thiserror::Error;
use uuid::Uuid;

pub(crate) trait HashFunc: Send + Sync {
    fn call(&self, password: &str) -> Result<String, HashError>;
}

impl<F> HashFunc for F
where
    F: Fn(&str) -> Result<String, HashError> + Send + Sync,
{
    fn call(&self, password: &str) -> Result<String, HashError> {
        self(password)
    }
}

pub(crate) fn default_hash_password(password: &str) -> Result<String, HashError> {
    let salt_string = SaltString::from_b64(&gen_uniq_b64_string()).map_err(|_| HashError::Salt)?;
    let salt = Salt::from(&salt_string);

    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), salt)
        .map_err(|_| HashError::Hash)?;
    Ok(password_hash.to_string())
}

fn gen_uniq_b64_string() -> String {
    BASE64_STANDARD.encode(Uuid::new_v4().to_string())
}

#[derive(Debug, Clone, Error)]
pub(crate) enum HashError {
    #[error("failed to create salt")]
    Salt,
    #[error("failed to hash password")]
    Hash,
}

fn verify_pass(password: &str, password_hash: &str) -> Result<(), HashError> {
    let password_hash = PasswordHash::try_from(password_hash).map_err(|_e| HashError::Hash)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &password_hash)
        .map_err(|_e| HashError::Hash)
}

#[cfg(test)]
mod test {
    use super::{default_hash_password, verify_pass};

    #[test]
    fn test_hash_password() {
        let password = "test_password";
        (default_hash_password(password)).unwrap();
    }

    #[test]
    fn test_hash_verify() {
        let password = "test_password2";
        let password_hash = (default_hash_password(password)).unwrap();

        verify_pass(password, &password_hash).expect("should same pass");
    }
}

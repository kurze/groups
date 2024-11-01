use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &[u8]) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(
    password: &[u8],
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password, &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = b"supersecret";
        let hashed = hash_password(password).expect("Failed to hash password");
        assert!(!hashed.is_empty(), "Hashed password should not be empty");
    }

    #[test]
    fn test_verify_password() {
        let password = b"supersecret";
        let hashed = hash_password(password).expect("Failed to hash password");
        let is_valid = verify_password(password, &hashed).expect("Failed to verify password");
        assert!(is_valid, "Password should be valid");
    }

    #[test]
    fn test_verify_password_invalid() {
        let password = b"supersecret";
        let hashed = hash_password(password).expect("Failed to hash password");
        let is_valid =
            verify_password(b"wrongpassword", &hashed).expect("Failed to verify password");
        assert!(!is_valid, "Password should be invalid");
    }
}

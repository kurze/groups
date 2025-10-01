use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Password validation errors
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password is too weak (score: {score}/4)")]
    TooWeak { score: u8 },

    #[error("Password validation failed: {0}")]
    ValidationFailed(String),

    #[error("Argon2 error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
}

/// Password strength feedback from zxcvbn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFeedback {
    /// Strength score (0=very weak, 4=very strong)
    pub score: u8,

    /// Warning about primary weakness (if any)
    pub warning: Option<String>,

    /// Specific suggestions to improve password
    pub suggestions: Vec<String>,

    /// Estimated time to crack
    pub crack_time_display: String,
}

/// Validate password strength using zxcvbn algorithm
///
/// # Arguments
/// * `password` - The password to validate
/// * `user_inputs` - User-specific data (email, name) to penalize weak passwords containing personal info
///
/// # Returns
/// * `Ok(entropy)` - Password strength estimation
/// * `Err(PasswordError)` - If validation fails
pub fn validate_password_strength(
    password: &str,
    user_inputs: &[&str],
) -> Result<zxcvbn::Entropy, PasswordError> {
    if password.is_empty() {
        return Err(PasswordError::ValidationFailed(
            "Password cannot be empty".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(PasswordError::ValidationFailed(
            "Password cannot exceed 128 characters".to_string(),
        ));
    }

    let entropy = zxcvbn::zxcvbn(password, user_inputs)
        .map_err(|e| PasswordError::ValidationFailed(e.to_string()))?;

    Ok(entropy)
}

/// Check if password meets minimum strength threshold
///
/// # Arguments
/// * `entropy` - Password strength estimation from zxcvbn
///
/// # Returns
/// * `true` if score >= 3 (safely unguessable)
/// * `false` if score < 3 (too weak)
pub fn is_password_strong_enough(entropy: &zxcvbn::Entropy) -> bool {
    // Require score of at least 3 (safely unguessable, moderate protection from offline attacks)
    // Score 0-2: Too weak
    // Score 3: Acceptable (moderate protection)
    // Score 4: Strong (very unguessable)
    entropy.score() >= 3
}

/// Format password feedback for user display
///
/// # Arguments
/// * `entropy` - Password strength estimation from zxcvbn
///
/// # Returns
/// * `PasswordFeedback` struct with score, warning, suggestions, and crack time
pub fn format_password_feedback(entropy: &zxcvbn::Entropy) -> PasswordFeedback {
    let feedback = entropy.feedback().as_ref();

    let warning = feedback.and_then(|f| f.warning()).map(|w| w.to_string());

    let suggestions = feedback
        .map(|f| f.suggestions().iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    PasswordFeedback {
        score: entropy.score(),
        warning,
        suggestions,
        crack_time_display: entropy.crack_times().offline_slow_hashing_1e4_per_second().to_string(),
    }
}

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

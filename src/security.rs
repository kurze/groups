use rand::Rng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Constant-time comparison of byte slices to prevent timing attacks
///
/// Uses the `subtle` crate's ConstantTimeEq trait to ensure comparison
/// takes the same amount of time regardless of where differences occur.
///
/// # Arguments
/// * `a` - First byte slice
/// * `b` - Second byte slice
///
/// # Returns
/// * `true` if slices are equal, `false` otherwise
///
/// # Security
/// This function is critical for preventing timing attacks in:
/// - Password hash comparisons
/// - Token validation
/// - Checking if user emails exist
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    // If lengths differ, still do constant-time comparison to avoid timing leaks
    if a.len() != b.len() {
        // Compare against empty slice of same length to maintain constant time
        let empty = vec![0u8; a.len()];
        return a.ct_eq(&empty).into();
    }

    a.ct_eq(b).into()
}

/// Generate cryptographically secure random token
///
/// Creates a 32-byte random token encoded as URL-safe base64.
/// Used for password reset tokens, session tokens, etc.
///
/// # Returns
/// * `String` - URL-safe base64 encoded random token
///
/// # Security
/// Uses `rand::thread_rng()` which provides cryptographically secure randomness.
/// 32 bytes = 256 bits of entropy, sufficient for security tokens.
pub fn generate_secure_token() -> String {
    let mut rng = rand::thread_rng();
    let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    // Use base64 URL-safe encoding (no padding) for tokens in URLs
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &token_bytes)
}

/// Hash token using SHA-256
///
/// Tokens are hashed before storage in the database to prevent token theft
/// if the database is compromised. Only the hashed version is stored.
///
/// # Arguments
/// * `token` - Plaintext token to hash
///
/// # Returns
/// * `String` - Hex-encoded SHA-256 hash of the token
///
/// # Security
/// SHA-256 is used (not Argon2) because:
/// - Tokens are already high-entropy random data (32 bytes)
/// - No need for slow hashing (defense is token randomness, not brute force resistance)
/// - Fast lookups in database are desirable
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_equal() {
        let a = b"hello";
        let b = b"hello";
        assert!(constant_time_eq(a, b));
    }

    #[test]
    fn test_constant_time_eq_not_equal() {
        let a = b"hello";
        let b = b"world";
        assert!(!constant_time_eq(a, b));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        let a = b"hello";
        let b = b"hi";
        assert!(!constant_time_eq(a, b));
    }

    #[test]
    fn test_generate_secure_token() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();

        // Tokens should be non-empty
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());

        // Tokens should be unique (extremely high probability)
        assert_ne!(token1, token2);

        // Token should be URL-safe (no padding, no special chars)
        assert!(!token1.contains('='));
        assert!(!token1.contains('+'));
        assert!(!token1.contains('/'));
    }

    #[test]
    fn test_hash_token() {
        let token = "test-token-123";
        let hash = hash_token(token);

        // SHA-256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same token should produce same hash
        let hash2 = hash_token(token);
        assert_eq!(hash, hash2);

        // Different tokens should produce different hashes
        let hash3 = hash_token("different-token");
        assert_ne!(hash, hash3);
    }
}

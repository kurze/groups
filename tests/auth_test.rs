use actix_web::{App, http::StatusCode, test};
use groups::password::{hash_password, verify_password};

#[actix_web::test]
async fn test_password_hashing() {
    let password = b"test_password_123";

    // Test hashing
    let hash = hash_password(password).expect("Failed to hash password");
    assert!(!hash.is_empty());

    // Test verification with correct password
    assert!(verify_password(password, &hash).expect("Failed to verify password"));

    // Test verification with incorrect password
    assert!(!verify_password(b"wrong_password", &hash).expect("Failed to verify password"));
}

#[actix_web::test]
async fn test_password_hash_uniqueness() {
    let password = b"test_password_123";

    // Hash the same password twice
    let hash1 = hash_password(password).expect("Failed to hash password");
    let hash2 = hash_password(password).expect("Failed to hash password");

    // Hashes should be different due to random salts
    assert_ne!(hash1, hash2);

    // Both should verify correctly
    assert!(verify_password(password, &hash1).expect("Failed to verify password"));
    assert!(verify_password(password, &hash2).expect("Failed to verify password"));
}

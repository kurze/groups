#[cfg(test)]
mod password_security_tests {
    use groups::password::{
        validate_password_strength, is_password_strong_enough, format_password_feedback,
        hash_password, verify_password,
    };
    use groups::security::{constant_time_eq, generate_secure_token, hash_token};

    // ==================== Password Strength Tests ====================

    #[test]
    fn test_weak_password_rejected() {
        let weak_passwords = vec![
            "password",
            "123456",
            "qwerty",
            "letmein",
            "admin",
        ];

        for password in weak_passwords {
            let result = validate_password_strength(password, &[]);
            assert!(result.is_ok(), "Password validation should not error: {}", password);

            let entropy = result.unwrap();
            assert!(
                !is_password_strong_enough(&entropy),
                "Weak password should be rejected: {} (score: {})",
                password,
                entropy.score()
            );
        }
    }

    #[test]
    fn test_strong_password_accepted() {
        let strong_passwords = vec![
            "Tr0ub4dor&3-Correct-Horse-Battery",
            "MyV3ry$ecur3P@ssw0rd!2024",
            "uncommon-phrase-with-numbers-42",
        ];

        for password in strong_passwords {
            let result = validate_password_strength(password, &[]);
            assert!(result.is_ok(), "Password validation should not error: {}", password);

            let entropy = result.unwrap();
            assert!(
                is_password_strong_enough(&entropy),
                "Strong password should be accepted: {} (score: {})",
                password,
                entropy.score()
            );
        }
    }

    #[test]
    fn test_password_with_user_data_penalized() {
        let email = "john.doe@example.com";
        let name = "John Doe";
        let user_inputs = [email, name];

        // Password containing user data should be weaker
        let weak_with_email = "johndoe123";
        let entropy_weak = validate_password_strength(weak_with_email, &user_inputs).unwrap();

        // Same pattern but without user data should be stronger
        let same_pattern = "janedoe123";
        let entropy_no_user = validate_password_strength(same_pattern, &[]).unwrap();

        assert!(
            entropy_weak.score() <= entropy_no_user.score(),
            "Password with user data should have equal or lower score"
        );
    }

    #[test]
    fn test_password_feedback_format() {
        let password = "password";
        let entropy = validate_password_strength(password, &[]).unwrap();
        let feedback = format_password_feedback(&entropy);

        assert_eq!(feedback.score, entropy.score());
        assert!(feedback.score < 3, "Weak password should have score < 3");
        // Feedback should provide suggestions
        assert!(
            feedback.warning.is_some() || !feedback.suggestions.is_empty(),
            "Weak password should have warnings or suggestions"
        );
    }

    #[test]
    fn test_empty_password_rejected() {
        let result = validate_password_strength("", &[]);
        assert!(result.is_err(), "Empty password should be rejected");
    }

    #[test]
    fn test_password_too_long_rejected() {
        let long_password = "a".repeat(129);
        let result = validate_password_strength(&long_password, &[]);
        assert!(result.is_err(), "Password over 128 characters should be rejected");
    }

    // ==================== Password Hashing Tests ====================

    #[test]
    fn test_password_hash_and_verify() {
        let password = b"my-secure-password";

        let hash = hash_password(password).expect("Should hash password");
        assert!(!hash.is_empty(), "Hash should not be empty");
        assert!(hash.starts_with("$argon2"), "Hash should be Argon2 format");

        let is_valid = verify_password(password, &hash).expect("Should verify password");
        assert!(is_valid, "Correct password should verify");

        let wrong_password = b"wrong-password";
        let is_invalid = verify_password(wrong_password, &hash).expect("Should verify password");
        assert!(!is_invalid, "Wrong password should not verify");
    }

    #[test]
    fn test_password_hash_uniqueness() {
        let password = b"same-password";

        let hash1 = hash_password(password).expect("Should hash password");
        let hash2 = hash_password(password).expect("Should hash password");

        // Same password should produce different hashes (due to random salt)
        assert_ne!(hash1, hash2, "Same password should produce different hashes");

        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    // ==================== Security Utilities Tests ====================

    #[test]
    fn test_constant_time_eq_equal() {
        let a = b"secret-token";
        let b = b"secret-token";
        assert!(constant_time_eq(a, b), "Equal values should return true");
    }

    #[test]
    fn test_constant_time_eq_not_equal() {
        let a = b"secret-token";
        let b = b"different-token";
        assert!(!constant_time_eq(a, b), "Different values should return false");
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        let a = b"short";
        let b = b"longer-value";
        assert!(!constant_time_eq(a, b), "Different lengths should return false");
    }

    #[test]
    fn test_generate_secure_token_uniqueness() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();

        assert!(!token1.is_empty(), "Token should not be empty");
        assert!(!token2.is_empty(), "Token should not be empty");
        assert_ne!(token1, token2, "Tokens should be unique");

        // Tokens should be URL-safe (no padding, no +, no /)
        assert!(!token1.contains('='), "Token should not contain padding");
        assert!(!token1.contains('+'), "Token should not contain +");
        assert!(!token1.contains('/'), "Token should not contain /");
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "test-token-123";

        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2, "Same token should produce same hash");
        assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 hex characters");
    }

    #[test]
    fn test_hash_token_uniqueness() {
        let token1 = "token-one";
        let token2 = "token-two";

        let hash1 = hash_token(token1);
        let hash2 = hash_token(token2);

        assert_ne!(hash1, hash2, "Different tokens should produce different hashes");
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_password_validation_flow() {
        // Simulate user registration with password validation
        let email = "user@example.com";
        let name = "Test User";
        let password = "MySecureP@ssw0rd2024!";

        // Validate password strength
        let entropy = validate_password_strength(password, &[email, name])
            .expect("Password validation should succeed");

        assert!(
            is_password_strong_enough(&entropy),
            "Password should be strong enough (score: {})",
            entropy.score()
        );

        // Hash the password
        let hash = hash_password(password.as_bytes())
            .expect("Password hashing should succeed");

        // Verify the password
        let is_valid = verify_password(password.as_bytes(), &hash)
            .expect("Password verification should succeed");

        assert!(is_valid, "Hashed password should verify correctly");
    }

    #[test]
    fn test_password_reset_token_flow() {
        // Generate a secure token
        let token = generate_secure_token();
        assert!(!token.is_empty(), "Token should be generated");

        // Hash it for storage
        let token_hash = hash_token(&token);
        assert_eq!(token_hash.len(), 64, "Token hash should be SHA-256");

        // Simulate lookup: hash incoming token and compare
        let incoming_token = token.clone();
        let incoming_hash = hash_token(&incoming_token);

        assert_eq!(
            token_hash, incoming_hash,
            "Hashed tokens should match for validation"
        );

        // Wrong token should not match
        let wrong_token = generate_secure_token();
        let wrong_hash = hash_token(&wrong_token);
        assert_ne!(token_hash, wrong_hash, "Different token should not match");
    }
}

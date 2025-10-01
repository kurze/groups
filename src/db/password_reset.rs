use crate::db::models::password_reset_token::PasswordResetToken;
use crate::security::{generate_secure_token, hash_token};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Password reset service errors
#[derive(Error, Debug)]
pub enum PasswordResetError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Token not found or invalid")]
    TokenNotFound,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Token has already been used")]
    TokenUsed,
}

/// Password reset token service
///
/// Manages password reset token lifecycle:
/// - Token generation and storage
/// - Token validation
/// - Token expiration (15 minutes)
/// - Single-use enforcement
pub struct PasswordResetService;

impl PasswordResetService {
    /// Create a password reset token
    ///
    /// Generates a cryptographically secure token and stores its hash.
    /// Tokens expire after 15 minutes and are single-use.
    ///
    /// # Arguments
    /// * `user_id` - User ID to create token for
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok((token_id, plaintext_token))` - Token ID and plaintext token to send via email
    /// * `Err(PasswordResetError)` - Database error
    ///
    /// # Security
    /// Only the hashed token is stored in the database.
    /// The plaintext token must be sent immediately via email and never logged.
    pub async fn create_reset_token(
        user_id: i32,
        pool: &PgPool,
    ) -> Result<(Uuid, String), PasswordResetError> {
        let plaintext_token = generate_secure_token();
        let token_hash = hash_token(&plaintext_token);
        let expires_at = Utc::now() + Duration::minutes(15);

        let token_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok((token_id, plaintext_token))
    }

    /// Validate and consume a password reset token
    ///
    /// Checks if token is valid, not expired, and not already used.
    /// Marks the token as used to prevent reuse.
    ///
    /// # Arguments
    /// * `plaintext_token` - Token from email link
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(user_id)` - Valid token, returns associated user ID
    /// * `Err(PasswordResetError)` - Token invalid, expired, or already used
    ///
    /// # Security
    /// This is a critical security operation - must validate all conditions.
    pub async fn validate_and_consume_token(
        plaintext_token: &str,
        pool: &PgPool,
    ) -> Result<i32, PasswordResetError> {
        let token_hash = hash_token(plaintext_token);

        // Fetch token record
        let token: PasswordResetToken = sqlx::query_as(
            r#"
            SELECT id, user_id, token_hash, expires_at, used_at, created_at
            FROM password_reset_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(pool)
        .await?
        .ok_or(PasswordResetError::TokenNotFound)?;

        // Validate token state
        if !token.is_valid() {
            if token.is_expired() {
                return Err(PasswordResetError::TokenExpired);
            } else if token.is_used() {
                return Err(PasswordResetError::TokenUsed);
            } else {
                return Err(PasswordResetError::TokenNotFound);
            }
        }

        // Mark token as used
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(token.id)
        .execute(pool)
        .await?;

        Ok(token.user_id)
    }

    /// Invalidate all existing password reset tokens for a user
    ///
    /// Called when:
    /// - User successfully resets password
    /// - User logs in successfully
    /// - User requests a new reset token
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of tokens invalidated
    /// * `Err(PasswordResetError)` - Database error
    pub async fn invalidate_user_tokens(
        user_id: i32,
        pool: &PgPool,
    ) -> Result<u64, PasswordResetError> {
        let result = sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE user_id = $1 AND used_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired tokens
    ///
    /// Removes expired tokens from the database to prevent table growth.
    /// Should be run periodically (e.g., daily cron job).
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of tokens deleted
    /// * `Err(PasswordResetError)` - Database error
    pub async fn cleanup_expired_tokens(pool: &PgPool) -> Result<u64, PasswordResetError> {
        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_reset_error_display() {
        assert_eq!(
            PasswordResetError::TokenExpired.to_string(),
            "Token has expired"
        );
        assert_eq!(
            PasswordResetError::TokenUsed.to_string(),
            "Token has already been used"
        );
        assert_eq!(
            PasswordResetError::TokenNotFound.to_string(),
            "Token not found or invalid"
        );
    }
}

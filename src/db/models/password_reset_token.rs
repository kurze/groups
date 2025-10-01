use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Password reset token for secure password reset flow
/// Tokens are single-use, time-limited (15 minutes), and stored hashed
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    /// Unique identifier for this token record
    pub id: Uuid,

    /// User ID this token belongs to
    pub user_id: i32,

    /// SHA-256 hash of the actual token (never store plaintext)
    pub token_hash: String,

    /// When this token expires (15 minutes after creation)
    pub expires_at: DateTime<Utc>,

    /// When this token was used (NULL = unused, prevents reuse)
    pub used_at: Option<DateTime<Utc>>,

    /// When this token was created
    pub created_at: DateTime<Utc>,
}

impl PasswordResetToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token has been used
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    /// Check if token is valid (not expired and not used)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

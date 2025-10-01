use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Rate limiting record for preventing brute force attacks
/// Tracks attempts per identifier (IP or email) and action type
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RateLimitRecord {
    /// Unique identifier for this record
    pub id: i32,

    /// Identifier being rate limited (IP address or email)
    pub identifier: String,

    /// Type of action being rate limited
    /// Examples: "login", "password_reset", "registration"
    pub action_type: String,

    /// Number of attempts in current window
    pub attempt_count: i32,

    /// Start of current rate limit window
    pub window_start: DateTime<Utc>,

    /// When next attempt is allowed (NULL = no delay, used for exponential backoff)
    pub next_allowed_at: Option<DateTime<Utc>>,
}

impl RateLimitRecord {
    /// Check if rate limit window has expired (1 hour window)
    pub fn is_window_expired(&self) -> bool {
        let now = Utc::now();
        let window_duration = Duration::hours(1);
        now.signed_duration_since(self.window_start) > window_duration
    }

    /// Check if currently rate limited (has active delay)
    pub fn is_rate_limited(&self) -> bool {
        if let Some(next_allowed) = self.next_allowed_at {
            Utc::now() < next_allowed
        } else {
            false
        }
    }

    /// Calculate exponential backoff delay in seconds
    /// Formula: 2^(attempts - threshold) seconds
    /// Example: attempts 6,7,8,9 = delays 1s, 2s, 4s, 8s
    pub fn calculate_backoff_delay(attempts: i32, threshold: i32) -> i64 {
        if attempts <= threshold {
            return 0;
        }
        let exponent = (attempts - threshold) as u32;
        2_i64.pow(exponent)
    }

    /// Get seconds until next attempt is allowed
    pub fn seconds_until_allowed(&self) -> Option<i64> {
        self.next_allowed_at.map(|next_allowed| {
            let now = Utc::now();
            (next_allowed - now).num_seconds().max(0)
        })
    }
}

/// Standard action types for rate limiting
pub mod action_types {
    pub const LOGIN: &str = "login";
    pub const PASSWORD_RESET: &str = "password_reset";
    pub const REGISTRATION: &str = "registration";
    pub const PASSWORD_CHANGE: &str = "password_change";
}

/// Rate limit thresholds per action type
pub mod thresholds {
    /// Login: 5 attempts before backoff starts
    pub const LOGIN: i32 = 5;

    /// Password reset: 3 requests per hour
    pub const PASSWORD_RESET: i32 = 3;

    /// Registration: 5 attempts per hour per IP
    pub const REGISTRATION: i32 = 5;

    /// Password change: 10 attempts per hour per session
    pub const PASSWORD_CHANGE: i32 = 10;
}

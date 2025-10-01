use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Authentication event log for security audit trail
/// Logs all auth attempts, password changes, and security events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthLog {
    /// Unique identifier for this log entry
    pub id: i32,

    /// User ID (NULL if user not found during attempt)
    pub user_id: Option<i32>,

    /// Type of authentication event
    /// Examples: "login_success", "login_failure", "password_reset_requested", "password_changed"
    pub event_type: String,

    /// Whether the event succeeded
    pub success: bool,

    /// IP address of the client
    pub ip_address: String,

    /// HTTP User-Agent header (optional)
    pub user_agent: Option<String>,

    /// Email address attempted (even if user not found, for audit)
    pub email_attempted: Option<String>,

    /// Error message if failed (never includes sensitive data)
    pub error_message: Option<String>,

    /// When this event occurred
    pub created_at: DateTime<Utc>,
}

/// Standard event types for authentication logging
pub mod event_types {
    pub const LOGIN_SUCCESS: &str = "login_success";
    pub const LOGIN_FAILURE: &str = "login_failure";
    pub const LOGIN_RATE_LIMITED: &str = "login_rate_limited";
    pub const PASSWORD_RESET_REQUESTED: &str = "password_reset_requested";
    pub const PASSWORD_RESET_COMPLETED: &str = "password_reset_completed";
    pub const PASSWORD_RESET_SUCCESS: &str = "password_reset_success";
    pub const PASSWORD_RESET_FAILURE: &str = "password_reset_failure";
    pub const PASSWORD_CHANGED: &str = "password_changed";
    pub const PASSWORD_CHANGE_FAILED: &str = "password_change_failed";
    pub const SESSION_EXPIRED: &str = "session_expired";
    pub const REGISTRATION_SUCCESS: &str = "registration_success";
    pub const REGISTRATION_FAILURE: &str = "registration_failure";
}

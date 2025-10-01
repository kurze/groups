use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Current User model for PostgreSQL
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    // Password security fields
    #[serde(default)]
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
}

impl User {
    /// Check if user account is currently locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Check if user has exceeded maximum failed login attempts
    pub fn has_exceeded_login_attempts(&self, max_attempts: i32) -> bool {
        self.failed_login_attempts >= max_attempts
    }
}

// Data transfer object for creating users
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
}

// Data transfer object for updating users
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub password_hash: Option<String>,
}

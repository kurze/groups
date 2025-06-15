use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

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
}

impl User {
    pub fn new(email: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0, // Will be set by database
            email,
            name: String::new(),
            password_hash: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
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

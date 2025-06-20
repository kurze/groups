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

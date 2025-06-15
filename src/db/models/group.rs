use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Current Group model for PostgreSQL
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}


// Data transfer object for creating groups
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateGroup {
    pub name: String,
}

// Data transfer object for updating groups
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateGroup {
    pub name: String,
}

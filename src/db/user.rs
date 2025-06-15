use crate::db::models::User;
use crate::db::connection::{DbPool, DatabaseError};
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum UserError {
    #[error("User must be soft-deleted before hard deletion")]
    NotSoftDeleted,
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DbError(#[from] DatabaseError),
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),
}

pub struct UserService {
    pool: DbPool,
}

#[allow(dead_code)]
impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // Create a new user
    pub async fn create(&self, email: String, name: String) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *"
        )
        .bind(&email)
        .bind(&name)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // Create a new user with password
    pub async fn create_with_password(&self, email: String, name: String, password_hash: String) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(&email)
        .bind(&name)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // Read user by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<User>, UserError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // Read user by email
    pub async fn get_by_email(&self, email: String) -> Result<Option<User>, UserError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(&email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // Update user
    pub async fn update(&self, user: User) -> Result<(), UserError> {
        let affected_rows = sqlx::query(
            "UPDATE users SET name = $1, password_hash = $2, updated_at = NOW() WHERE id = $3 AND deleted_at IS NULL"
        )
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(user.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected_rows == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    // Delete user by ID (soft delete)
    pub async fn delete(&self, id: i32) -> Result<(), UserError> {
        sqlx::query(
            "UPDATE users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Hard delete user by ID (permanent removal)
    // Only allowed for users that have already been soft-deleted
    pub async fn hard_delete(&self, id: i32) -> Result<(), UserError> {
        // First check if user exists and is soft-deleted
        let user = self.get_by_id(id).await?;
        
        match user {
            Some(user) => {
                // Check if the user is soft-deleted
                if user.deleted_at.is_none() {
                    return Err(UserError::NotSoftDeleted);
                }

                // User is soft-deleted, proceed with hard deletion
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
                
                Ok(())
            }
            None => Ok(()), // User doesn't exist, nothing to delete
        }
    }

    // List all active users
    pub async fn list_active(&self) -> Result<Vec<User>, UserError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    // Count total number of users
    pub async fn count(&self) -> Result<i64, UserError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}

// Tests will be rewritten for PostgreSQL in a separate module

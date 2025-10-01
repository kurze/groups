use crate::db::connection::{DatabaseError, DbPool};
use crate::db::models::User;
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
            "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *",
        )
        .bind(&email)
        .bind(&name)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // Create a new user with password
    pub async fn create_with_password(
        &self,
        email: String,
        name: String,
        password_hash: String,
    ) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING *",
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
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    // Read user by email
    pub async fn get_by_email(&self, email: String) -> Result<Option<User>, UserError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
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
        sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
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
            "SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC",
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

    /// Increment failed login attempts for a user
    ///
    /// Increments the failed login counter and optionally locks the account
    /// if maximum attempts are exceeded.
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(new_count)` - New failed attempt count
    /// * `Err(UserError)` - Database error
    pub async fn increment_failed_login(
        user_id: i32,
        pool: &sqlx::PgPool,
    ) -> Result<i32, UserError> {
        let (new_count,): (i32,) = sqlx::query_as(
            r#"
            UPDATE users
            SET failed_login_attempts = failed_login_attempts + 1,
                locked_until = CASE
                    WHEN failed_login_attempts + 1 >= 5 THEN NOW() + INTERVAL '15 minutes'
                    ELSE locked_until
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING failed_login_attempts
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(new_count)
    }

    /// Reset failed login attempts after successful login
    ///
    /// Clears the failed login counter and updates last login information.
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `ip` - IP address of successful login
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(())` - Reset successful
    /// * `Err(UserError)` - Database error
    pub async fn reset_failed_login_attempts(
        user_id: i32,
        ip: &str,
        pool: &sqlx::PgPool,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_attempts = 0,
                locked_until = NULL,
                last_login_at = NOW(),
                last_login_ip = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(ip)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Change user's password
    ///
    /// Updates the password hash for a user.
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `new_password_hash` - New Argon2id password hash
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(())` - Password changed successfully
    /// * `Err(UserError)` - Database error or user not found
    pub async fn change_password(
        user_id: i32,
        new_password_hash: String,
        pool: &sqlx::PgPool,
    ) -> Result<(), UserError> {
        let affected_rows = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(&new_password_hash)
        .execute(pool)
        .await?
        .rows_affected();

        if affected_rows == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }
}

// Tests will be rewritten for PostgreSQL in a separate module

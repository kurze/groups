use sqlx::PgPool;
use thiserror::Error;

/// Cleanup task errors
#[derive(Error, Debug)]
pub enum CleanupError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Periodic cleanup tasks for maintaining database health
///
/// These tasks should be run periodically (e.g., every hour or daily)
/// to prevent unbounded growth of security-related tables.
pub struct CleanupTasks;

impl CleanupTasks {
    /// Clean up expired password reset tokens
    ///
    /// Removes tokens that expired more than 24 hours ago.
    /// This prevents the password_reset_tokens table from growing indefinitely.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of tokens deleted
    /// * `Err(CleanupError)` - Database error
    pub async fn cleanup_expired_reset_tokens(pool: &PgPool) -> Result<u64, CleanupError> {
        let result = sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < NOW() - INTERVAL '24 hours'
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up old authentication logs
    ///
    /// Removes auth logs older than 90 days.
    /// This balances audit requirements with database storage.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of logs deleted
    /// * `Err(CleanupError)` - Database error
    ///
    /// # Note
    /// Adjust the 90-day retention based on your compliance requirements.
    pub async fn cleanup_old_auth_logs(pool: &PgPool) -> Result<u64, CleanupError> {
        let result = sqlx::query(
            r#"
            DELETE FROM auth_logs
            WHERE created_at < NOW() - INTERVAL '90 days'
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up stale rate limit records
    ///
    /// Removes rate limit records where the window expired more than 24 hours ago.
    /// This prevents the rate_limit_records table from growing indefinitely.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of records deleted
    /// * `Err(CleanupError)` - Database error
    pub async fn cleanup_stale_rate_limits(pool: &PgPool) -> Result<u64, CleanupError> {
        let result = sqlx::query(
            r#"
            DELETE FROM rate_limit_records
            WHERE window_start < NOW() - INTERVAL '24 hours'
              AND (next_allowed_at IS NULL OR next_allowed_at < NOW() - INTERVAL '24 hours')
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Run all cleanup tasks
    ///
    /// Convenience method to run all cleanup tasks in sequence.
    /// Logs results and continues even if individual tasks fail.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * Total number of records deleted across all tasks
    pub async fn run_all(pool: &PgPool) -> u64 {
        let mut total_deleted = 0;

        match Self::cleanup_expired_reset_tokens(pool).await {
            Ok(count) => {
                if count > 0 {
                    println!("Cleanup: Deleted {} expired password reset tokens", count);
                }
                total_deleted += count;
            }
            Err(e) => {
                eprintln!("Cleanup error (reset tokens): {}", e);
            }
        }

        match Self::cleanup_old_auth_logs(pool).await {
            Ok(count) => {
                if count > 0 {
                    println!("Cleanup: Deleted {} old auth logs (>90 days)", count);
                }
                total_deleted += count;
            }
            Err(e) => {
                eprintln!("Cleanup error (auth logs): {}", e);
            }
        }

        match Self::cleanup_stale_rate_limits(pool).await {
            Ok(count) => {
                if count > 0 {
                    println!("Cleanup: Deleted {} stale rate limit records", count);
                }
                total_deleted += count;
            }
            Err(e) => {
                eprintln!("Cleanup error (rate limits): {}", e);
            }
        }

        if total_deleted > 0 {
            println!("Cleanup: Total {} records deleted", total_deleted);
        }

        total_deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_error_display() {
        let err = CleanupError::Database(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
    }
}

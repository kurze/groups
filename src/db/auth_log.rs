use chrono::{DateTime, Utc};
use sqlx::PgPool;
use thiserror::Error;

/// Auth logging service errors
#[derive(Error, Debug)]
pub enum AuthLogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Authentication logging service
///
/// Provides audit trail for security events including:
/// - Login attempts (success and failure)
/// - Password changes
/// - Password reset requests
/// - Registration attempts
/// - Account lockouts
pub struct AuthLogService;

impl AuthLogService {
    /// Log an authentication event
    ///
    /// # Arguments
    /// * `user_id` - User ID (None for failed login attempts where user doesn't exist)
    /// * `event_type` - Type of event (use constants from db::models::auth_log::event_types)
    /// * `success` - Whether the event was successful
    /// * `ip` - IP address of the client
    /// * `user_agent` - User agent string (optional)
    /// * `email` - Email attempted (for failed logins, optional)
    /// * `error` - Error message (for failures, optional)
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(())` - Event logged successfully
    /// * `Err(AuthLogError)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// use crate::db::models::auth_log::event_types;
    ///
    /// AuthLogService::log_auth_event(
    ///     Some(user_id),
    ///     event_types::LOGIN_SUCCESS,
    ///     true,
    ///     "192.168.1.1",
    ///     Some("Mozilla/5.0..."),
    ///     None,
    ///     None,
    ///     &pool
    /// ).await?;
    /// ```
    pub async fn log_auth_event(
        user_id: Option<i32>,
        event_type: &str,
        success: bool,
        ip: &str,
        user_agent: Option<&str>,
        email: Option<&str>,
        error: Option<&str>,
        pool: &PgPool,
    ) -> Result<(), AuthLogError> {
        sqlx::query(
            r#"
            INSERT INTO auth_logs (user_id, event_type, success, ip_address, user_agent, email_attempted, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(event_type)
        .bind(success)
        .bind(ip)
        .bind(user_agent)
        .bind(email)
        .bind(error)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get recent auth logs for a user
    ///
    /// Retrieves the most recent authentication events for a user.
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `limit` - Maximum number of logs to retrieve
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(Vec<(event_type, success, ip, timestamp)>)` - List of recent events
    /// * `Err(AuthLogError)` - Database error
    pub async fn get_recent_logs(
        user_id: i32,
        limit: i32,
        pool: &PgPool,
    ) -> Result<Vec<(String, bool, String, DateTime<Utc>)>, AuthLogError> {
        let logs = sqlx::query_as::<_, (String, bool, String, DateTime<Utc>)>(
            r#"
            SELECT event_type, success, ip_address, created_at
            FROM auth_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Get failed login attempts for an email
    ///
    /// Retrieves recent failed login attempts for security monitoring.
    ///
    /// # Arguments
    /// * `email` - Email address
    /// * `since` - Only count attempts after this time
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(count)` - Number of failed attempts
    /// * `Err(AuthLogError)` - Database error
    pub async fn count_failed_login_attempts(
        email: &str,
        since: DateTime<Utc>,
        pool: &PgPool,
    ) -> Result<i64, AuthLogError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM auth_logs
            WHERE email_attempted = $1
              AND event_type = 'login_failure'
              AND success = false
              AND created_at >= $2
            "#,
        )
        .bind(email)
        .bind(since)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for auth log service (integration tests require database)

    #[test]
    fn test_auth_log_error_display() {
        let err = AuthLogError::Database(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
    }
}

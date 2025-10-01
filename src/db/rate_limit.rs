use crate::db::models::rate_limit::{action_types, thresholds, RateLimitRecord};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use thiserror::Error;

/// Rate limit service errors
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub attempts_remaining: i32,
    pub retry_after_seconds: Option<i64>,
}

/// Rate limiting service
///
/// Implements sliding window rate limiting using PostgreSQL.
/// Uses exponential backoff for repeated violations.
pub struct RateLimitService;

impl RateLimitService {
    /// Check if action is rate limited
    ///
    /// # Arguments
    /// * `identifier` - IP address or email to rate limit
    /// * `action_type` - Type of action (login, password_reset, registration)
    /// * `threshold` - Maximum attempts allowed in the time window
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(RateLimitStatus)` - Rate limit check result
    /// * `Err(RateLimitError)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// let status = RateLimitService::check_rate_limit(
    ///     "192.168.1.1",
    ///     action_types::LOGIN,
    ///     thresholds::LOGIN,
    ///     &pool
    /// ).await?;
    ///
    /// if !status.allowed {
    ///     return Err("Rate limit exceeded");
    /// }
    /// ```
    pub async fn check_rate_limit(
        identifier: &str,
        action_type: &str,
        threshold: i32,
        pool: &PgPool,
    ) -> Result<RateLimitStatus, RateLimitError> {
        // Try to get existing rate limit record
        let record: Option<RateLimitRecord> = sqlx::query_as(
            r#"
            SELECT id, identifier, action_type, attempt_count, window_start, next_allowed_at, created_at, updated_at
            FROM rate_limit_records
            WHERE identifier = $1 AND action_type = $2
            "#,
        )
        .bind(identifier)
        .bind(action_type)
        .fetch_optional(pool)
        .await?;

        match record {
            None => {
                // No record exists, allow the action
                Ok(RateLimitStatus {
                    allowed: true,
                    attempts_remaining: threshold - 1,
                    retry_after_seconds: None,
                })
            }
            Some(record) => {
                // Check if we're in a backoff period
                if let Some(next_allowed) = record.next_allowed_at {
                    if Utc::now() < next_allowed {
                        let retry_after = (next_allowed - Utc::now()).num_seconds();
                        return Ok(RateLimitStatus {
                            allowed: false,
                            attempts_remaining: 0,
                            retry_after_seconds: Some(retry_after),
                        });
                    }
                }

                // Check if window has expired (5 minutes)
                if record.is_window_expired() {
                    // Window expired, reset and allow
                    Ok(RateLimitStatus {
                        allowed: true,
                        attempts_remaining: threshold - 1,
                        retry_after_seconds: None,
                    })
                } else if record.is_rate_limited(threshold) {
                    // Within window and over threshold
                    let retry_after = (record.window_start + Duration::minutes(5) - Utc::now())
                        .num_seconds()
                        .max(0);
                    Ok(RateLimitStatus {
                        allowed: false,
                        attempts_remaining: 0,
                        retry_after_seconds: Some(retry_after),
                    })
                } else {
                    // Within window, under threshold
                    Ok(RateLimitStatus {
                        allowed: true,
                        attempts_remaining: threshold - record.attempt_count - 1,
                        retry_after_seconds: None,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for rate limit logic (integration tests require database)

    #[test]
    fn test_rate_limit_status_creation() {
        let status = RateLimitStatus {
            allowed: true,
            attempts_remaining: 4,
            retry_after_seconds: None,
        };
        assert!(status.allowed);
        assert_eq!(status.attempts_remaining, 4);
        assert!(status.retry_after_seconds.is_none());
    }

    #[test]
    fn test_rate_limit_status_blocked() {
        let status = RateLimitStatus {
            allowed: false,
            attempts_remaining: 0,
            retry_after_seconds: Some(60),
        };
        assert!(!status.allowed);
        assert_eq!(status.attempts_remaining, 0);
        assert_eq!(status.retry_after_seconds, Some(60));
    }
}

use crate::db::rate_limit::{RateLimitError, RateLimitService};
use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use sqlx::PgPool;
use std::future::{ready, Ready};
use std::rc::Rc;

/// Rate limiting middleware configuration
///
/// Protects endpoints from brute force attacks using configurable
/// action types and thresholds.
#[derive(Clone)]
pub struct RateLimit {
    action_type: String,
    threshold: i32,
}

impl RateLimit {
    /// Create new rate limiting middleware
    ///
    /// # Arguments
    /// * `action_type` - Action to rate limit (login, password_reset, registration)
    /// * `threshold` - Maximum attempts allowed in 5-minute window
    pub fn new(action_type: impl Into<String>, threshold: i32) -> Self {
        Self {
            action_type: action_type.into(),
            threshold,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware {
            service: Rc::new(service),
            action_type: self.action_type.clone(),
            threshold: self.threshold,
        }))
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    action_type: String,
    threshold: i32,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let action_type = self.action_type.clone();
        let threshold = self.threshold;

        Box::pin(async move {
            // Extract IP address from connection info
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("unknown")
                .to_string();

            // Get database pool from app data
            let pool = match req.app_data::<actix_web::web::Data<PgPool>>() {
                Some(pool) => pool.get_ref().clone(),
                None => {
                    return Ok(req.into_response(
                        HttpResponse::InternalServerError()
                            .body("Database pool not configured")
                            .map_into_boxed_body(),
                    ));
                }
            };

            // Check rate limit
            match RateLimitService::check_rate_limit(&ip, &action_type, threshold, &pool).await {
                Ok(status) => {
                    if !status.allowed {
                        let retry_after = status.retry_after_seconds.unwrap_or(60);
                        return Ok(req.into_response(
                            HttpResponse::TooManyRequests()
                                .insert_header(("Retry-After", retry_after.to_string()))
                                .body(format!(
                                    "Rate limit exceeded. Try again in {} seconds.",
                                    retry_after
                                ))
                                .map_into_boxed_body(),
                        ));
                    }
                }
                Err(RateLimitError::Database(e)) => {
                    // Log error but don't block request on database failures
                    eprintln!("Rate limit check failed: {}", e);
                }
                Err(RateLimitError::RateLimitExceeded) => {
                    return Ok(req.into_response(
                        HttpResponse::TooManyRequests()
                            .body("Rate limit exceeded")
                            .map_into_boxed_body(),
                    ));
                }
            }

            // Continue to next middleware/handler
            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_creation() {
        let rate_limit = RateLimit::new("login", 5);
        assert_eq!(rate_limit.action_type, "login");
        assert_eq!(rate_limit.threshold, 5);
    }

    #[test]
    fn test_rate_limit_string_conversion() {
        let rate_limit = RateLimit::new("password_reset".to_string(), 3);
        assert_eq!(rate_limit.action_type, "password_reset");
        assert_eq!(rate_limit.threshold, 3);
    }
}

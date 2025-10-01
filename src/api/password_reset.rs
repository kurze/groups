use crate::db::auth_log::AuthLogService;
use crate::db::models::event_types;
use crate::db::models::rate_limit::{action_types, thresholds};
use crate::db::password_reset::PasswordResetService;
use crate::db::rate_limit::RateLimitService;
use crate::db::user::UserService;
use crate::email::EmailService;
use crate::password::{validate_password_strength, is_password_strong_enough, format_password_feedback, hash_password};
use crate::security::constant_time_eq;
use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestPayload {
    email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmPayload {
    token: String,
    new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetRequestResponse {
    message: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetConfirmResponse {
    message: String,
}

/// POST /api/auth/password-reset/request
///
/// Request a password reset email. Uses constant-time response to prevent
/// user enumeration attacks.
pub async fn password_reset_request_api(
    payload: web::Json<PasswordResetRequestPayload>,
    pool: web::Data<PgPool>,
    email_service: web::Data<EmailService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit
    let rate_limit_status = RateLimitService::check_rate_limit(
        &ip,
        action_types::PASSWORD_RESET,
        thresholds::PASSWORD_RESET,
        &pool,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Rate limit error: {}", e)))?;

    if !rate_limit_status.allowed {
        let retry_after = rate_limit_status.retry_after_seconds.unwrap_or(60);
        return Ok(HttpResponse::TooManyRequests()
            .insert_header(("Retry-After", retry_after.to_string()))
            .json(PasswordResetRequestResponse {
                message: format!("Too many requests. Try again in {} seconds.", retry_after),
            }));
    }

    // Record the attempt
    RateLimitService::record_attempt(&ip, action_types::PASSWORD_RESET, &pool)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Rate limit error: {}", e)))?;

    // Look up user by email
    let user_service = UserService::new(pool.get_ref().clone());
    let user_result = user_service
        .get_by_email(payload.email.clone())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    // Use constant-time response to prevent user enumeration
    let mut email_sent = false;

    if let Some(user) = user_result {
        // User exists, create reset token
        match PasswordResetService::create_reset_token(user.id, &pool).await {
            Ok((_token_id, plaintext_token)) => {
                // Send reset email
                match email_service.send_password_reset_email(
                    &user.email,
                    &plaintext_token,
                    &user.name,
                ) {
                    Ok(_) => {
                        email_sent = true;
                        // Log successful reset request
                        let _ = AuthLogService::log_auth_event(
                            Some(user.id),
                            event_types::PASSWORD_RESET_REQUESTED,
                            true,
                            &ip,
                            None,
                            None,
                            None,
                            &pool,
                        )
                        .await;
                    }
                    Err(e) => {
                        eprintln!("Failed to send reset email: {}", e);
                        // Log failure
                        let _ = AuthLogService::log_auth_event(
                            Some(user.id),
                            event_types::PASSWORD_RESET_REQUESTED,
                            false,
                            &ip,
                            None,
                            None,
                            Some(&format!("Email send failed: {}", e)),
                            &pool,
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create reset token: {}", e);
            }
        }
    } else {
        // User doesn't exist - simulate processing time for constant-time response
        let dummy_email = "nonexistent@example.com";
        let _ = constant_time_eq(dummy_email.as_bytes(), payload.email.as_bytes());
    }

    // Always return success to prevent user enumeration
    Ok(HttpResponse::Ok().json(PasswordResetRequestResponse {
        message: "If an account exists with that email, a password reset link has been sent.".to_string(),
    }))
}

/// POST /api/auth/password-reset/confirm
///
/// Confirm password reset with token and new password.
pub async fn password_reset_confirm_api(
    payload: web::Json<PasswordResetConfirmPayload>,
    pool: web::Data<PgPool>,
    email_service: web::Data<EmailService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Validate and consume token
    let user_id = match PasswordResetService::validate_and_consume_token(&payload.token, &pool).await {
        Ok(user_id) => user_id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(PasswordResetConfirmResponse {
                message: format!("Invalid or expired token: {}", e),
            }));
        }
    };

    // Get user for validation context
    let user_service = UserService::new(pool.get_ref().clone());
    let user = user_service
        .get_by_id(user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    // Validate password strength
    let user_inputs = [user.email.as_str(), user.name.as_str()];
    let entropy = validate_password_strength(&payload.new_password, &user_inputs)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Password validation failed: {}", e)))?;

    if !is_password_strong_enough(&entropy) {
        let feedback = format_password_feedback(&entropy);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Password is too weak",
            "feedback": feedback,
        })));
    }

    // Hash new password
    let new_password_hash = hash_password(payload.new_password.as_bytes())
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Password hashing failed: {}", e)))?;

    // Update password
    UserService::change_password(user_id, new_password_hash, &pool)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Password update failed: {}", e)))?;

    // Invalidate any remaining reset tokens for this user
    let _ = PasswordResetService::invalidate_user_tokens(user_id, &pool).await;

    // Send confirmation email
    let _ = email_service.send_password_changed_notification(&user.email, &user.name);

    // Log successful password reset
    let _ = AuthLogService::log_auth_event(
        Some(user_id),
        event_types::PASSWORD_RESET_COMPLETED,
        true,
        &ip,
        None,
        None,
        None,
        &pool,
    )
    .await;

    Ok(HttpResponse::Ok().json(PasswordResetConfirmResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}

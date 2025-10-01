use crate::db::auth_log::AuthLogService;
use crate::db::models::event_types;
use crate::db::password_reset::PasswordResetService;
use crate::db::user::UserService;
use crate::email::EmailService;
use crate::password::{validate_password_strength, is_password_strong_enough, format_password_feedback, hash_password, verify_password};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct PasswordChangePayload {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordChangeResponse {
    message: String,
}

/// POST /api/auth/password/change
///
/// Change authenticated user's password. Requires current password verification.
pub async fn password_change_api(
    payload: web::Json<PasswordChangePayload>,
    pool: web::Data<PgPool>,
    email_service: web::Data<EmailService>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Get user from session
    let user_id: i32 = session
        .get("user_id")
        .map_err(|_| actix_web::error::ErrorUnauthorized("Session error"))?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    // Get user from database
    let user_service = UserService::new(pool.get_ref().clone());
    let user = user_service
        .get_by_id(user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    // Verify current password
    let current_hash = user.password_hash.as_ref()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("No password set"))?;

    let password_valid = verify_password(payload.current_password.as_bytes(), current_hash)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Password verification failed: {}", e)))?;

    if !password_valid {
        // Log failed password change attempt
        let _ = AuthLogService::log_auth_event(
            Some(user_id),
            event_types::PASSWORD_CHANGE_FAILED,
            false,
            &ip,
            None,
            None,
            Some("Incorrect current password"),
            &pool,
        )
        .await;

        return Ok(HttpResponse::BadRequest().json(PasswordChangeResponse {
            message: "Current password is incorrect".to_string(),
        }));
    }

    // Validate new password strength
    let user_inputs = [user.email.as_str(), user.name.as_str()];
    let entropy = validate_password_strength(&payload.new_password, &user_inputs)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Password validation failed: {}", e)))?;

    if !is_password_strong_enough(&entropy) {
        let feedback = format_password_feedback(&entropy);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "New password is too weak",
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

    // Invalidate any password reset tokens
    let _ = PasswordResetService::invalidate_user_tokens(user_id, &pool).await;

    // Send notification email
    let _ = email_service.send_password_changed_notification(&user.email, &user.name);

    // Log successful password change
    let _ = AuthLogService::log_auth_event(
        Some(user_id),
        event_types::PASSWORD_CHANGED,
        true,
        &ip,
        None,
        None,
        None,
        &pool,
    )
    .await;

    Ok(HttpResponse::Ok().json(PasswordChangeResponse {
        message: "Password changed successfully".to_string(),
    }))
}

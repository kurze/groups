use super::{hash_password, verify_password};
use crate::db::auth_log::AuthLogService;
use crate::db::models::auth_log::event_types;
use crate::db::rate_limit::{RateLimitService, action_types, thresholds};
use crate::db::user::UserService;
use crate::password::{validate_password_strength, is_password_strong_enough, format_password_feedback};
use actix_session::Session;
use actix_web::{HttpResponse, Result, web, HttpRequest};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tera::Tera;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

pub async fn login_page(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let ctx = tera::Context::new();
    let rendered = tmpl.render("login.html", &ctx).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub async fn login(
    form: web::Form<LoginRequest>,
    user_service: web::Data<UserService>,
    pool: web::Data<PgPool>,
    session: Session,
    req: HttpRequest,
    _tmpl: web::Data<Tera>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit before processing
    let rate_limit_status = RateLimitService::check_rate_limit(
        &ip,
        action_types::LOGIN,
        thresholds::LOGIN,
        &pool,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Rate limit error: {}", e)))?;

    if !rate_limit_status.allowed {
        let retry_after = rate_limit_status.retry_after_seconds.unwrap_or(60);
        ctx.insert("message", &format!("Too many login attempts. Try again in {} seconds.", retry_after));
        ctx.insert("success", &false);

        // Return early with rate limit message
        let fragment = r#"
            <div id="login-form">
                <div class="alert alert-error">{{ message }}</div>

                <div class="form-group">
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" required autocomplete="email">
                </div>

                <div class="form-group">
                    <label for="password">Password:</label>
                    <input type="password" id="password" name="password" required autocomplete="current-password">
                </div>

                <div class="form-actions">
                    <button type="submit">Login</button>
                </div>

                <div class="form-links">
                    <a href="/register">Don't have an account? Register</a>
                </div>
            </div>
        "#;

        let rendered = Tera::one_off(fragment, &ctx, false).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
        })?;

        return Ok(HttpResponse::Ok().content_type("text/html").body(rendered));
    }

    // Look up user by email
    match user_service.get_by_email(form.email.clone()).await {
        Ok(Some(user)) => {
            // Check if account is locked
            if user.is_locked() {
                // Record failed attempt for rate limiting
                let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;

                // Log locked account attempt
                let _ = AuthLogService::log_auth_event(
                    Some(user.id),
                    event_types::LOGIN_FAILURE,
                    false,
                    &ip,
                    req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                    Some(&form.email),
                    Some("Account locked"),
                    &pool,
                ).await;

                ctx.insert("message", "Account is temporarily locked. Please try again later.");
                ctx.insert("success", &false);
            } else if let Some(password_hash) = &user.password_hash {
                match verify_password(form.password.as_bytes(), password_hash) {
                    Ok(true) => {
                        // Password is correct - reset failed attempts and update login info
                        let _ = UserService::reset_failed_login_attempts(user.id, &ip, &pool).await;
                        let _ = RateLimitService::reset_rate_limit(&ip, action_types::LOGIN, &pool).await;

                        // Store user info in session
                        session.insert("user_id", user.id).unwrap();
                        session.insert("user_email", &user.email).unwrap();
                        session.insert("user_name", &user.name).unwrap();
                        session.insert("created_at", Utc::now()).unwrap();
                        session.insert("last_activity", Utc::now()).unwrap();

                        // Log successful login
                        let _ = AuthLogService::log_auth_event(
                            Some(user.id),
                            event_types::LOGIN_SUCCESS,
                            true,
                            &ip,
                            req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                            None,
                            None,
                            &pool,
                        ).await;

                        ctx.insert("message", "Login successful!");
                        ctx.insert("success", &true);
                    }
                    Ok(false) => {
                        // Password is incorrect - increment failed attempts
                        let _ = UserService::increment_failed_login(user.id, &pool).await;
                        let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;

                        // Log failed login
                        let _ = AuthLogService::log_auth_event(
                            Some(user.id),
                            event_types::LOGIN_FAILURE,
                            false,
                            &ip,
                            req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                            Some(&form.email),
                            Some("Invalid password"),
                            &pool,
                        ).await;

                        ctx.insert("message", "Invalid email or password");
                        ctx.insert("success", &false);
                    }
                    Err(_) => {
                        // Error verifying password
                        let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;
                        ctx.insert("message", "Authentication error");
                        ctx.insert("success", &false);
                    }
                }
            } else {
                // User has no password set (legacy user)
                let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;
                ctx.insert("message", "Please reset your password");
                ctx.insert("success", &false);
            }
        }
        Ok(None) => {
            // User not found - record attempt and use constant time to prevent enumeration
            let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;

            // Log failed login attempt
            let _ = AuthLogService::log_auth_event(
                None,
                event_types::LOGIN_FAILURE,
                false,
                &ip,
                req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                Some(&form.email),
                Some("User not found"),
                &pool,
            ).await;

            ctx.insert("message", "Invalid email or password");
            ctx.insert("success", &false);
        }
        Err(_) => {
            // Database error
            let _ = RateLimitService::record_attempt(&ip, action_types::LOGIN, &pool).await;
            ctx.insert("message", "Authentication error");
            ctx.insert("success", &false);
        }
    }

    // Return just the form fragment for htmz to replace
    let fragment = r#"
        <div id="login-form">
            {% if success %}
                <div class="alert alert-success">{{ message }}</div>
                <script>setTimeout(() => window.top.location.href = '/groups', 1500);</script>
            {% else %}
                <div class="alert alert-error">{{ message }}</div>
            {% endif %}
            
            <div class="form-group">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required autocomplete="email">
            </div>
            
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required autocomplete="current-password">
            </div>
            
            <div class="form-actions">
                <button type="submit">Login</button>
            </div>
            
            <div class="form-links">
                <a href="/register">Don't have an account? Register</a>
            </div>
        </div>
    "#;

    let rendered = Tera::one_off(fragment, &ctx, false).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub async fn register_page(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let ctx = tera::Context::new();
    let rendered = tmpl.render("register.html", &ctx).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub async fn register(
    form: web::Form<RegisterRequest>,
    user_service: web::Data<UserService>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
    _tmpl: web::Data<Tera>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();

    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit
    let rate_limit_status = RateLimitService::check_rate_limit(
        &ip,
        action_types::REGISTRATION,
        thresholds::REGISTRATION,
        &pool,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Rate limit error: {}", e)))?;

    if !rate_limit_status.allowed {
        let retry_after = rate_limit_status.retry_after_seconds.unwrap_or(60);
        ctx.insert("message", &format!("Too many registration attempts. Try again in {} seconds.", retry_after));
        ctx.insert("success", &false);

        // Return early with rate limit message
        let fragment = r#"
            <div id="register-form">
                <div class="alert alert-error">{{ message }}</div>

                <div class="form-group">
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" required autocomplete="email">
                </div>

                <div class="form-group">
                    <label for="password">Password:</label>
                    <input type="password" id="password" name="password" required autocomplete="new-password">
                </div>

                <div class="form-actions">
                    <button type="submit">Register</button>
                </div>

                <div class="form-links">
                    <a href="/login">Already have an account? Login</a>
                </div>
            </div>
        "#;

        let rendered = Tera::one_off(fragment, &ctx, false).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
        })?;

        return Ok(HttpResponse::Ok().content_type("text/html").body(rendered));
    }

    // Check if email already exists
    match user_service.get_by_email(form.email.clone()).await {
        Ok(Some(_)) => {
            // User already exists
            let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;
            ctx.insert("message", "Email already registered");
            ctx.insert("success", &false);
        }
        Ok(None) => {
            // Validate password strength
            let name = form.email.split('@').next().unwrap_or("User").to_string();
            let user_inputs = [form.email.as_str(), name.as_str()];

            match validate_password_strength(&form.password, &user_inputs) {
                Ok(entropy) => {
                    if !is_password_strong_enough(&entropy) {
                        // Password too weak
                        let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;
                        let feedback = format_password_feedback(&entropy);

                        let mut message = format!("Password is too weak (score: {}/4). ", feedback.score);
                        if let Some(warning) = &feedback.warning {
                            message.push_str(&format!("Warning: {}. ", warning));
                        }
                        if !feedback.suggestions.is_empty() {
                            message.push_str(&format!("Suggestions: {}", feedback.suggestions.join(", ")));
                        }

                        ctx.insert("message", &message);
                        ctx.insert("success", &false);
                    } else {
                        // Password is strong enough, create user
                        match hash_password(form.password.as_bytes()) {
                            Ok(password_hash) => {
                                match user_service
                                    .create_with_password(form.email.clone(), name.clone(), password_hash)
                                    .await
                                {
                                    Ok(user) => {
                                        // Log successful registration
                                        let _ = AuthLogService::log_auth_event(
                                            Some(user.id),
                                            event_types::REGISTRATION_SUCCESS,
                                            true,
                                            &ip,
                                            req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                                            None,
                                            None,
                                            &pool,
                                        ).await;

                                        ctx.insert("message", "Registration successful! Please login.");
                                        ctx.insert("success", &true);
                                    }
                                    Err(e) => {
                                        let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;

                                        // Log failed registration
                                        let _ = AuthLogService::log_auth_event(
                                            None,
                                            event_types::REGISTRATION_FAILURE,
                                            false,
                                            &ip,
                                            req.headers().get("user-agent").and_then(|h| h.to_str().ok()),
                                            Some(&form.email),
                                            Some(&format!("Database error: {}", e)),
                                            &pool,
                                        ).await;

                                        ctx.insert("message", &format!("Registration failed: {}", e));
                                        ctx.insert("success", &false);
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;
                                ctx.insert("message", "Failed to process password");
                                ctx.insert("success", &false);
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;
                    ctx.insert("message", &format!("Password validation failed: {}", e));
                    ctx.insert("success", &false);
                }
            }
        }
        Err(e) => {
            let _ = RateLimitService::record_attempt(&ip, action_types::REGISTRATION, &pool).await;
            ctx.insert("message", &format!("Registration failed: {}", e));
            ctx.insert("success", &false);
        }
    }

    // Return just the form fragment for htmz to replace
    let fragment = r#"
        <div id="register-form">
            {% if success %}
                <div class="alert alert-success">{{ message }}</div>
                <script>setTimeout(() => window.top.location.href = '/login', 1500);</script>
            {% else %}
                <div class="alert alert-error">{{ message }}</div>
            {% endif %}
            
            <div class="form-group">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required autocomplete="email">
            </div>
            
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required autocomplete="new-password" minlength="8">
            </div>
            
            <div class="form-actions">
                <button type="submit">Register</button>
            </div>
            
            <div class="form-links">
                <a href="/login">Already have an account? Login</a>
            </div>
        </div>
    "#;

    let rendered = Tera::one_off(fragment, &ctx, false).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    // Clear the session
    session.clear();

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}

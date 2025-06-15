use super::{hash_password, verify_password};
use crate::db::user::UserService;
use actix_session::Session;
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    success: bool,
    message: String,
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
    session: Session,
    _tmpl: web::Data<Tera>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();

    // Look up user by email
    match user_service.get_by_email(form.email.clone()).await {
        Ok(Some(user)) => {
            // User exists, check password
            if let Some(password_hash) = &user.password_hash {
                match verify_password(form.password.as_bytes(), password_hash) {
                    Ok(true) => {
                        // Password is correct
                        // Store user info in session
                        session.insert("user_id", user.id).unwrap();
                        session.insert("user_email", &user.email).unwrap();
                        session.insert("user_name", &user.name).unwrap();

                        ctx.insert("message", "Login successful!");
                        ctx.insert("success", &true);
                    }
                    Ok(false) => {
                        // Password is incorrect
                        ctx.insert("message", "Invalid email or password");
                        ctx.insert("success", &false);
                    }
                    Err(_) => {
                        // Error verifying password
                        ctx.insert("message", "Authentication error");
                        ctx.insert("success", &false);
                    }
                }
            } else {
                // User has no password set (legacy user)
                ctx.insert("message", "Please reset your password");
                ctx.insert("success", &false);
            }
        }
        Ok(None) => {
            // User not found
            ctx.insert("message", "Invalid email or password");
            ctx.insert("success", &false);
        }
        Err(_) => {
            // Database error
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
    _tmpl: web::Data<Tera>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();

    // Check if email already exists
    match user_service.get_by_email(form.email.clone()).await {
        Ok(Some(_)) => {
            // User already exists
            ctx.insert("message", "Email already registered");
            ctx.insert("success", &false);
        }
        Ok(None) => {
            // Create new user
            let name = form.email.split('@').next().unwrap_or("User").to_string();

            // Hash the password
            match hash_password(form.password.as_bytes()) {
                Ok(password_hash) => {
                    match user_service
                        .create_with_password(form.email.clone(), name, password_hash)
                        .await
                    {
                        Ok(_) => {
                            ctx.insert("message", "Registration successful! Please login.");
                            ctx.insert("success", &true);
                        }
                        Err(e) => {
                            ctx.insert("message", &format!("Registration failed: {}", e));
                            ctx.insert("success", &false);
                        }
                    }
                }
                Err(_) => {
                    ctx.insert("message", "Failed to process password");
                    ctx.insert("success", &false);
                }
            }
        }
        Err(e) => {
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

use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
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

pub async fn login(form: web::Form<LoginRequest>, _tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    // TODO: Implement actual authentication logic
    // For now, just return a success/failure message

    let mut ctx = tera::Context::new();

    if form.email == "test@example.com" && form.password == "password" {
        // Success - return a success fragment
        ctx.insert("message", "Login successful!");
        ctx.insert("success", &true);
    } else {
        // Failure - return an error fragment
        ctx.insert("message", "Invalid email or password");
        ctx.insert("success", &false);
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
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use thiserror::Error;

/// Email service errors
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Failed to build email message: {0}")]
    MessageBuild(#[from] lettre::error::Error),

    #[error("Failed to send email: {0}")]
    SendFailed(#[from] lettre::transport::smtp::Error),

    #[error("Missing environment variable: {0}")]
    MissingConfig(String),
}

/// Email service for sending notifications
///
/// Handles SMTP configuration and provides methods for sending
/// password-related email notifications.
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    from_email: String,
    from_name: String,
}

impl EmailService {
    /// Create a new EmailService from environment variables
    ///
    /// Required environment variables:
    /// - SMTP_HOST: SMTP server hostname
    /// - SMTP_PORT: SMTP server port
    /// - SMTP_FROM_EMAIL: Sender email address
    /// - SMTP_FROM_NAME: Sender display name
    ///
    /// Optional environment variables:
    /// - SMTP_USERNAME: SMTP authentication username
    /// - SMTP_PASSWORD: SMTP authentication password
    pub fn from_env() -> Result<Self, EmailError> {
        let smtp_host = env::var("SMTP_HOST")
            .map_err(|_| EmailError::MissingConfig("SMTP_HOST".to_string()))?;

        let smtp_port = env::var("SMTP_PORT")
            .map_err(|_| EmailError::MissingConfig("SMTP_PORT".to_string()))?
            .parse()
            .map_err(|_| EmailError::MissingConfig("SMTP_PORT (invalid number)".to_string()))?;

        let from_email = env::var("SMTP_FROM_EMAIL")
            .map_err(|_| EmailError::MissingConfig("SMTP_FROM_EMAIL".to_string()))?;

        let from_name = env::var("SMTP_FROM_NAME")
            .map_err(|_| EmailError::MissingConfig("SMTP_FROM_NAME".to_string()))?;

        let smtp_username = env::var("SMTP_USERNAME").ok();
        let smtp_password = env::var("SMTP_PASSWORD").ok();

        Ok(EmailService {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        })
    }

    /// Send an email message
    ///
    /// Internal helper method for sending emails via SMTP.
    fn send_email(&self, to_email: &str, subject: &str, body: &str) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())?;

        let mailer = if let (Some(username), Some(password)) =
            (&self.smtp_username, &self.smtp_password)
        {
            // SMTP with authentication
            let creds = Credentials::new(username.clone(), password.clone());
            SmtpTransport::relay(&self.smtp_host)?
                .port(self.smtp_port)
                .credentials(creds)
                .build()
        } else {
            // SMTP without authentication (local development)
            SmtpTransport::builder_dangerous(&self.smtp_host)
                .port(self.smtp_port)
                .build()
        };

        mailer.send(&email)?;
        Ok(())
    }

    /// Send password reset email
    ///
    /// Sends an email with a password reset link to the user.
    ///
    /// # Arguments
    /// * `to_email` - Recipient email address
    /// * `reset_token` - Plaintext password reset token (will be URL-encoded)
    /// * `user_name` - User's name for personalization
    ///
    /// # Security
    /// The reset link expires in 15 minutes (enforced by database).
    /// Token should be cryptographically random (use security::generate_secure_token).
    pub fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
        user_name: &str,
    ) -> Result<(), EmailError> {
        let reset_url = format!(
            "http://localhost:8080/reset-password?token={}",
            urlencoding::encode(reset_token)
        );

        let body = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #007bff; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Password Reset Request</h2>
        <p>Hello {},</p>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <a href="{}" class="button">Reset Password</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #007bff;">{}</p>
        <p><strong>This link will expire in 15 minutes.</strong></p>
        <p>If you didn't request a password reset, you can safely ignore this email. Your password will not be changed.</p>
        <div class="footer">
            <p>This is an automated message from Groups Platform. Please do not reply to this email.</p>
        </div>
    </div>
</body>
</html>
"#,
            user_name, reset_url, reset_url
        );

        self.send_email(to_email, "Reset Your Password", &body)
    }

    /// Send password changed notification
    ///
    /// Sends a security notification when a user's password is changed.
    /// This helps users detect unauthorized account access.
    ///
    /// # Arguments
    /// * `to_email` - Recipient email address
    /// * `user_name` - User's name for personalization
    ///
    /// # Security
    /// Always send this notification on password changes to alert users
    /// of potential unauthorized access.
    pub fn send_password_changed_notification(
        &self,
        to_email: &str,
        user_name: &str,
    ) -> Result<(), EmailError> {
        let body = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .alert {{ background-color: #d4edda; border: 1px solid #c3e6cb; border-radius: 4px; padding: 15px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border: 1px solid #ffeaa7; border-radius: 4px; padding: 15px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Password Changed Successfully</h2>
        <p>Hello {},</p>
        <div class="alert">
            <p><strong>Your password has been changed.</strong></p>
            <p>This is a confirmation that your account password was successfully updated.</p>
        </div>
        <div class="warning">
            <p><strong>Didn't make this change?</strong></p>
            <p>If you did not change your password, your account may have been compromised. Please:</p>
            <ul>
                <li>Reset your password immediately</li>
                <li>Review recent account activity</li>
                <li>Contact support if you need assistance</li>
            </ul>
        </div>
        <div class="footer">
            <p>This is an automated security notification from Groups Platform. Please do not reply to this email.</p>
        </div>
    </div>
</body>
</html>
"#,
            user_name
        );

        self.send_email(to_email, "Password Changed - Groups Platform", &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_service_from_env_missing_config() {
        // Clear environment variables
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_PORT");
        env::remove_var("SMTP_FROM_EMAIL");
        env::remove_var("SMTP_FROM_NAME");

        let result = EmailService::from_env();
        assert!(result.is_err());

        if let Err(EmailError::MissingConfig(var)) = result {
            assert_eq!(var, "SMTP_HOST");
        } else {
            panic!("Expected MissingConfig error");
        }
    }

    #[test]
    fn test_email_service_from_env_success() {
        // Set required environment variables
        env::set_var("SMTP_HOST", "localhost");
        env::set_var("SMTP_PORT", "1025");
        env::set_var("SMTP_FROM_EMAIL", "noreply@test.local");
        env::set_var("SMTP_FROM_NAME", "Test Service");

        let result = EmailService::from_env();
        assert!(result.is_ok());

        let service = result.unwrap();
        assert_eq!(service.smtp_host, "localhost");
        assert_eq!(service.smtp_port, 1025);
        assert_eq!(service.from_email, "noreply@test.local");
        assert_eq!(service.from_name, "Test Service");

        // Clean up
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_PORT");
        env::remove_var("SMTP_FROM_EMAIL");
        env::remove_var("SMTP_FROM_NAME");
    }
}

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

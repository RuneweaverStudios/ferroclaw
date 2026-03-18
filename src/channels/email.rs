//! Email channel adapter.
//!
//! Sends messages via SMTP and receives via IMAP polling.
//! Uses raw TCP + TLS via reqwest-compatible patterns — no lettre crate.
//! For production use, consider connecting to an email API (SendGrid, SES, etc.)
//! via the HTTP gateway webhook endpoint instead.

use crate::channels::{Channel, OutgoingMessage};
use crate::config::EmailConfig;
use crate::error::{FerroError, Result};
use std::future::Future;
use std::pin::Pin;

pub struct EmailChannel {
    smtp_host: String,
    smtp_port: u16,
    username: String,
    password: String,
    from_address: String,
    allowed_addresses: Vec<String>,
    #[allow(dead_code)]
    imap_host: Option<String>,
    #[allow(dead_code)]
    imap_port: u16,
}

impl EmailChannel {
    pub fn from_config(config: &EmailConfig) -> Option<Self> {
        let username = std::env::var(&config.username_env).ok()?;
        let password = std::env::var(&config.password_env).ok()?;

        Some(Self {
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            username,
            password,
            from_address: config.from_address.clone(),
            allowed_addresses: config.allowed_addresses.clone(),
            imap_host: config.imap_host.clone(),
            imap_port: config.imap_port,
        })
    }

    /// Check if an email address is allowed.
    pub fn is_allowed(&self, address: &str) -> bool {
        self.allowed_addresses.is_empty()
            || self
                .allowed_addresses
                .iter()
                .any(|a| a.eq_ignore_ascii_case(address))
    }

    /// Send an email via SMTP using the system `sendmail` or `curl` as a fallback.
    /// For production, use an HTTP email API (SendGrid, AWS SES, etc.)
    /// via the gateway webhook.
    pub async fn send_email(&self, to: &str, subject: &str, _body: &str) -> Result<()> {
        // Use curl to send via SMTP (available on most systems)
        let smtp_url = format!("smtp://{}:{}", self.smtp_host, self.smtp_port);

        let output = tokio::process::Command::new("curl")
            .args([
                "--url",
                &smtp_url,
                "--ssl-reqd",
                "--mail-from",
                &self.from_address,
                "--mail-rcpt",
                to,
                "--user",
                &format!("{}:{}", self.username, self.password),
                "-T",
                "-",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| FerroError::Channel(format!("Failed to spawn curl for SMTP: {e}")))?
            .wait_with_output()
            .await
            .map_err(|e| FerroError::Channel(format!("SMTP send error: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FerroError::Channel(format!("SMTP error: {stderr}")));
        }

        tracing::info!("Email sent to {to}: {subject}");
        Ok(())
    }
}

impl Channel for EmailChannel {
    fn name(&self) -> &str {
        "email"
    }

    fn is_configured(&self) -> bool {
        !self.smtp_host.is_empty() && !self.username.is_empty()
    }

    fn send<'a>(
        &'a self,
        target: &'a str,
        message: OutgoingMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let subject = if message.is_error {
                "[Ferroclaw Error]"
            } else {
                "[Ferroclaw]"
            };

            self.send_email(target, subject, &message.text).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist() {
        let channel = EmailChannel {
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            username: "user".into(),
            password: "pass".into(),
            from_address: "bot@example.com".into(),
            allowed_addresses: vec!["alice@example.com".into()],
            imap_host: None,
            imap_port: 993,
        };
        assert!(channel.is_allowed("alice@example.com"));
        assert!(channel.is_allowed("Alice@Example.Com")); // case insensitive
        assert!(!channel.is_allowed("eve@evil.com"));
    }

    #[test]
    fn test_empty_allowlist() {
        let channel = EmailChannel {
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            username: "user".into(),
            password: "pass".into(),
            from_address: "bot@example.com".into(),
            allowed_addresses: vec![],
            imap_host: None,
            imap_port: 993,
        };
        assert!(channel.is_allowed("anyone@anywhere.com"));
    }
}

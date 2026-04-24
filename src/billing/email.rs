//! Email service for sending invoices and receipts

use super::models::*;
use super::{BillingError, BillingResult};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// Configuration for email service
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_email: String,
        from_name: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        }
    }
}

/// Email message
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body: EmailBody,
    pub attachments: Vec<EmailAttachment>,
}

impl EmailMessage {
    pub fn new(to: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            to_name: None,
            subject: subject.into(),
            body: EmailBody::Text(String::new()),
            attachments: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.to_name = Some(name.into());
        self
    }

    pub fn with_html_body(mut self, body: impl Into<String>) -> Self {
        self.body = EmailBody::Html(body.into());
        self
    }

    pub fn with_text_body(mut self, body: impl Into<String>) -> Self {
        self.body = EmailBody::Text(body.into());
        self
    }

    pub fn with_attachment(mut self, attachment: EmailAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}

/// Email body type
#[derive(Debug, Clone)]
pub enum EmailBody {
    Text(String),
    Html(String),
}

/// Email attachment
#[derive(Debug, Clone)]
pub struct EmailAttachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl EmailAttachment {
    pub fn new(filename: impl Into<String>, content: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content,
            content_type: content_type.into(),
        }
    }

    pub fn from_path(path: &Path) -> BillingResult<Self> {
        let content = std::fs::read(path)
            .map_err(|e| BillingError::Email(format!("Failed to read attachment: {}", e)))?;

        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_string();

        Ok(Self::new(filename, content, content_type))
    }
}

/// Email service
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Send an email
    pub fn send_email(&self, message: &EmailMessage) -> BillingResult<()> {
        let mut email_builder = Message::builder()
            .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()
                .map_err(|e| BillingError::Email(format!("Invalid from address: {}", e)))?);

        if let Some(name) = &message.to_name {
            email_builder = email_builder.to(format!("{} <{}>", name, message.to).parse()
                .map_err(|e| BillingError::Email(format!("Invalid to address: {}", e)))?);
        } else {
            email_builder = email_builder.to(message.to.parse()
                .map_err(|e| BillingError::Email(format!("Invalid to address: {}", e)))?);
        }

        let email_body = match &message.body {
            EmailBody::Text(text) => {
                email_builder
                    .subject(&message.subject)
                    .header(ContentType::TEXT_PLAIN)
                    .body(text.clone())
            }
            EmailBody::Html(html) => {
                email_builder
                    .subject(&message.subject)
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone())
            }
        };

        let email = email_body
            .map_err(|e| BillingError::Email(format!("Failed to build email: {}", e)))?;

        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let mailer = SmtpTransport::relay(&self.config.smtp_host)
            .map_err(|e| BillingError::Email(format!("Failed to create mailer: {}", e)))?
            .credentials(creds)
            .build();

        mailer.send(&email)
            .map_err(|e| BillingError::Email(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    /// Send an invoice email
    pub fn send_invoice(
        &self,
        customer: &Customer,
        invoice: &Invoice,
        invoice_html: String,
        invoice_pdf_path: Option<&Path>,
    ) -> BillingResult<()> {
        let subject = format!("Invoice #{} from Ferroclaw", invoice.number);
        
        let body = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Invoice #{}</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #333;">Invoice #{}</h1>
        <p>Dear {},</p>
        <p>Your invoice is ready. Here are the details:</p>
        
        <div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0;">
            <p><strong>Invoice Number:</strong> {}</p>
            <p><strong>Issue Date:</strong> {}</p>
            <p><strong>Due Date:</strong> {}</p>
            <p><strong>Total:</strong> {:.2} {}</p>
        </div>

        <p>You can view the complete invoice details below.</p>
        {}

        <p style="margin-top: 30px; color: #666; font-size: 14px;">
            If you have any questions, please don't hesitate to contact us.
        </p>
        
        <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
        
        <p style="color: #999; font-size: 12px;">
            Ferroclaw - A Security-First AI Agent
        </p>
    </div>
</body>
</html>
"#,
            invoice.number,
            invoice.number,
            customer.name,
            invoice.number,
            invoice.created_at.format("%Y-%m-%d"),
            invoice.due_date.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            invoice.amount_cents as f64 / 100.0,
            invoice.currency,
            invoice_html
        );

        let mut email = EmailMessage::new(customer.email.clone(), subject)
            .with_name(customer.name.clone())
            .with_html_body(body);

        // Attach PDF if provided
        if let Some(pdf_path) = invoice_pdf_path {
            let attachment = EmailAttachment::from_path(pdf_path)?;
            email.attachments.push(attachment);
        }

        self.send_email(&email)
    }

    /// Send a payment receipt email
    pub fn send_receipt(
        &self,
        customer: &Customer,
        payment: &Payment,
        invoice: Option<&Invoice>,
    ) -> BillingResult<()> {
        let subject = "Payment Receipt - Ferroclaw";
        
        let invoice_details = if let Some(inv) = invoice {
            format!(
                r#"
<p>Invoice Details:</p>
<div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 10px 0;">
    <p><strong>Invoice Number:</strong> {}</p>
    <p><strong>Amount Paid:</strong> {:.2} {}</p>
    <p><strong>Payment Date:</strong> {}</p>
</div>
"#,
                inv.number,
                inv.amount_paid_cents as f64 / 100.0,
                inv.currency,
                inv.paid_at.map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            )
        } else {
            String::new()
        };

        let body = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Payment Receipt</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <div style="text-align: center; margin-bottom: 30px;">
            <div style="font-size: 48px; color: #4CAF50;">✓</div>
            <h1 style="color: #333;">Payment Received</h1>
        </div>
        
        <p>Dear {},</p>
        <p>Thank you for your payment! We're pleased to confirm that your payment has been processed successfully.</p>
        
        <div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0;">
            <p><strong>Payment ID:</strong> {}</p>
            <p><strong>Amount:</strong> {:.2} {}</p>
            <p><strong>Status:</strong> {}</p>
            <p><strong>Date:</strong> {}</p>
        </div>

        {}

        <p style="margin-top: 30px; color: #666; font-size: 14px;">
            If you have any questions about this payment, please don't hesitate to contact us.
        </p>
        
        <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
        
        <p style="color: #999; font-size: 12px;">
            Ferroclaw - A Security-First AI Agent
        </p>
    </div>
</body>
</html>
"#,
            customer.name,
            payment.id,
            payment.amount_cents as f64 / 100.0,
            payment.currency,
            format!("{:?}", payment.status),
            payment.created_at.format("%Y-%m-%d"),
            invoice_details
        );

        let email = EmailMessage::new(customer.email.clone(), subject)
            .with_name(customer.name.clone())
            .with_html_body(body);

        self.send_email(&email)
    }

    /// Send a subscription renewal reminder
    pub fn send_renewal_reminder(
        &self,
        customer: &Customer,
        subscription: &Subscription,
        days_until_renewal: u32,
    ) -> BillingResult<()> {
        let subject = format!("Subscription Renewal in {} Days - Ferroclaw", days_until_renewal);
        
        let body = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Subscription Renewal</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #333;">Subscription Renewal Reminder</h1>
        
        <p>Dear {},</p>
        <p>This is a friendly reminder that your subscription will be renewed in <strong>{} days</strong>.</p>
        
        <div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0;">
            <p><strong>Subscription ID:</strong> {}</p>
            <p><strong>Status:</strong> {}</p>
            <p><strong>Current Period Ends:</strong> {}</p>
            <p><strong>Next Billing Date:</strong> {}</p>
        </div>

        <p>If you need to make any changes to your subscription, you can do so from your account settings.</p>
        
        <p style="margin-top: 30px; color: #666; font-size: 14px;">
            If you have any questions, please don't hesitate to contact us.
        </p>
        
        <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
        
        <p style="color: #999; font-size: 12px;">
            Ferroclaw - A Security-First AI Agent
        </p>
    </div>
</body>
</html>
"#,
            customer.name,
            days_until_renewal,
            subscription.id,
            format!("{:?}", subscription.status),
            subscription.current_period_end.format("%Y-%m-%d"),
            subscription.current_period_end.format("%Y-%m-%d")
        );

        let email = EmailMessage::new(customer.email.clone(), subject)
            .with_name(customer.name.clone())
            .with_html_body(body);

        self.send_email(&email)
    }

    /// Send a payment failed email
    pub fn send_payment_failed(
        &self,
        customer: &Customer,
        invoice: &Invoice,
        retry_date: Option<DateTime<Utc>>,
    ) -> BillingResult<()> {
        let subject = "Payment Failed - Action Required - Ferroclaw";
        
        let retry_info = if let Some(date) = retry_date {
            format!(
                r#"<p style="color: #e65100;"><strong>Automatic retry scheduled for:</strong> {}</p>"#,
                date.format("%Y-%m-%d")
            )
        } else {
            String::new()
        };

        let body = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Payment Failed</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <div style="text-align: center; margin-bottom: 30px;">
            <div style="font-size: 48px; color: #f44336;">✕</div>
            <h1 style="color: #333;">Payment Failed</h1>
        </div>
        
        <p>Dear {},</p>
        <p>We were unable to process your payment. This could be due to an expired card, insufficient funds, or a temporary issue with your bank.</p>
        
        <div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0;">
            <p><strong>Invoice Number:</strong> {}</p>
            <p><strong>Amount Due:</strong> {:.2} {}</p>
            <p><strong>Due Date:</strong> {}</p>
        </div>

        {}

        <p style="margin-top: 30px;"><strong>What to do:</strong></p>
        <ul style="margin-top: 10px;">
            <li>Log in to your account to update your payment method</li>
            <li>Contact your bank if you suspect an issue with your card</li>
            <li>Reply to this email if you need assistance</li>
        </ul>
        
        <p style="margin-top: 30px; color: #666; font-size: 14px;">
            Please take action soon to avoid any interruption to your service.
        </p>
        
        <hr style="margin: 30px 0; border: none; border-top: 1px solid #eee;">
        
        <p style="color: #999; font-size: 12px;">
            Ferroclaw - A Security-First AI Agent
        </p>
    </div>
</body>
</html>
"#,
            customer.name,
            invoice.number,
            invoice.amount_due_cents as f64 / 100.0,
            invoice.currency,
            invoice.due_date.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            retry_info
        );

        let email = EmailMessage::new(customer.email.clone(), subject)
            .with_name(customer.name.clone())
            .with_html_body(body);

        self.send_email(&email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_builder() {
        let message = EmailMessage::new("test@example.com", "Test Subject")
            .with_name("Test User")
            .with_text_body("Test body");

        assert_eq!(message.to, "test@example.com");
        assert_eq!(message.subject, "Test Subject");
        assert_eq!(message.to_name, Some("Test User".to_string()));
    }
}

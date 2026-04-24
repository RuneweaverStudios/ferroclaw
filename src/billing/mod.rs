//! Billing system for Ferroclaw
//! 
//! This module provides comprehensive billing functionality including:
//! - Payment gateway integration (Stripe)
//! - PDF invoice generation
//! - Email service for invoices and receipts
//! - Customer dashboard data
//! - Proration calculations
//! - Webhook handling
//! - Payment retry mechanisms

pub mod client;
pub mod invoice;
pub mod email;
pub mod proration;
pub mod webhook;
pub mod retry;
pub mod models;

pub use client::{BillingClient, BillingConfig};
pub use invoice::{InvoiceGenerator, InvoiceData};
pub use email::{EmailService, EmailConfig, EmailMessage};
pub use proration::{ProrationCalculator, ProrationResult};
pub use webhook::{WebhookHandler, WebhookEvent};
pub use retry::{RetryScheduler, RetryConfig};
pub use models::{
    Customer, Subscription, Plan, Invoice, Payment, BillingError,
    BillingResult, PaymentStatus, SubscriptionStatus, InvoiceStatus
};

use thiserror::Error;

/// Unified billing error type
#[derive(Error, Debug)]
pub enum BillingError {
    #[error("Payment gateway error: {0}")]
    PaymentGateway(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Invoice generation error: {0}")]
    InvoiceGeneration(String),
    
    #[error("Email sending error: {0}")]
    Email(String),
    
    #[error("Proration calculation error: {0}")]
    Proration(String),
    
    #[error("Webhook processing error: {0}")]
    Webhook(String),
    
    #[error("Retry scheduling error: {0}")]
    Retry(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type BillingResult<T> = Result<T, BillingError>;

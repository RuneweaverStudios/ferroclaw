//! Core data models for the billing system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Customer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub stripe_customer_id: Option<String>,
    pub email: String,
    pub name: String,
    pub billing_address: Option<Address>,
    pub tax_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub stripe_price_id: String,
    pub name: String,
    pub description: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub interval: BillingInterval,
    pub trial_days: Option<u32>,
    pub metadata: serde_json::Value,
}

/// Billing interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingInterval {
    Monthly,
    Yearly,
}

/// Customer subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub plan_id: Uuid,
    pub stripe_subscription_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub cancel_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Unpaid,
    Incomplete,
    IncompleteExpired,
}

impl SubscriptionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Trialing)
    }
    
    pub fn is_past_due(&self) -> bool {
        matches!(self, Self::PastDue | Self::Unpaid)
    }
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub stripe_invoice_id: Option<String>,
    pub customer_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub number: String,
    pub status: InvoiceStatus,
    pub amount_cents: i64,
    pub amount_paid_cents: i64,
    pub amount_due_cents: i64,
    pub currency: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub line_items: Vec<InvoiceLineItem>,
    pub prorations: Vec<ProrationItem>,
    pub pdf_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub amount_cents: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Proration item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProrationItem {
    pub id: Uuid,
    pub description: String,
    pub amount_cents: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub proration_type: ProrationType,
}

/// Proration type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProrationType {
    Upgrade,
    Downgrade,
    Cancellation,
    Credit,
}

/// Payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub stripe_payment_intent_id: String,
    pub customer_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub description: Option<String>,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    Succeeded,
    Canceled,
    Failed,
}

impl PaymentStatus {
    pub fn is_pending(&self) -> bool {
        matches!(
            self,
            Self::RequiresPaymentMethod
                | Self::RequiresConfirmation
                | Self::RequiresAction
                | Self::Processing
        )
    }
    
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Canceled | Self::Failed)
    }
}

//! Payment gateway client (Stripe integration)

use super::models::*;
use super::{BillingError, BillingResult};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Configuration for the billing client
#[derive(Debug, Clone)]
pub struct BillingConfig {
    pub stripe_secret_key: String,
    pub webhook_secret: Option<String>,
    pub api_base: String,
}

impl BillingConfig {
    pub fn new(stripe_secret_key: String) -> Self {
        Self {
            stripe_secret_key,
            webhook_secret: None,
            api_base: "https://api.stripe.com".to_string(),
        }
    }
    
    pub fn with_webhook_secret(mut self, webhook_secret: String) -> Self {
        self.webhook_secret = Some(webhook_secret);
        self
    }
}

/// Payment gateway client
pub struct BillingClient {
    config: Arc<BillingConfig>,
    http_client: Client,
}

impl BillingClient {
    /// Create a new billing client
    pub fn new(config: BillingConfig) -> Self {
        Self {
            config: Arc::new(config),
            http_client: Client::new(),
        }
    }
    
    /// Create a new customer in Stripe
    pub async fn create_customer(&self, customer_data: &CustomerData) -> BillingResult<Customer> {
        let url = format!("{}/v1/customers", self.config.api_base);
        
        let mut params = vec![
            ("email", customer_data.email.as_str()),
            ("name", customer_data.name.as_str()),
        ];
        
        if let Some(description) = &customer_data.description {
            params.push(("description", description.as_str()));
        }
        
        let stripe_customer: StripeCustomer = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        Ok(Customer {
            id: Uuid::new_v4(),
            stripe_customer_id: Some(stripe_customer.id),
            email: customer_data.email.clone(),
            name: customer_data.name.clone(),
            billing_address: None,
            tax_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    /// Create a subscription
    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
        trial_days: Option<u32>,
    ) -> BillingResult<Subscription> {
        let url = format!("{}/v1/subscriptions", self.config.api_base);
        
        let mut params = vec![
            ("customer", customer_id),
            ("items[0][price]", price_id),
        ];
        
        if let Some(days) = trial_days {
            params.push(("trial_period_days", &days.to_string()));
        }
        
        let stripe_subscription: StripeSubscription = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        self.map_stripe_subscription(stripe_subscription)
    }
    
    /// Update a subscription
    pub async fn update_subscription(
        &self,
        subscription_id: &str,
        price_id: &str,
        proration_behavior: ProrationBehavior,
    ) -> BillingResult<Subscription> {
        let url = format!("{}/v1/subscriptions/{}", self.config.api_base, subscription_id);
        
        let params = vec![
            ("items[0][id]", subscription_id),
            ("items[0][price]", price_id),
            ("proration_behavior", &proration_behavior.as_str()),
        ];
        
        let stripe_subscription: StripeSubscription = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        self.map_stripe_subscription(stripe_subscription)
    }
    
    /// Cancel a subscription
    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> BillingResult<Subscription> {
        let url = format!("{}/v1/subscriptions/{}", self.config.api_base, subscription_id);
        
        let params = vec![("cancel_at_period_end", &at_period_end.to_string())];
        
        let stripe_subscription: StripeSubscription = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        self.map_stripe_subscription(stripe_subscription)
    }
    
    /// Create an invoice
    pub async fn create_invoice(
        &self,
        customer_id: &str,
        description: Option<String>,
    ) -> BillingResult<Invoice> {
        let url = format!("{}/v1/invoices", self.config.api_base);
        
        let mut params = vec![("customer", customer_id)];
        
        if let Some(desc) = description {
            params.push(("description", desc.as_str()));
        }
        
        let stripe_invoice: StripeInvoice = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        self.map_stripe_invoice(stripe_invoice)
    }
    
    /// Finalize and pay an invoice
    pub async fn pay_invoice(&self, invoice_id: &str) -> BillingResult<Invoice> {
        let url = format!(
            "{}/v1/invoices/{}/pay",
            self.config.api_base, invoice_id
        );
        
        let stripe_invoice: StripeInvoice = self
            .http_client
            .post(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        self.map_stripe_invoice(stripe_invoice)
    }
    
    /// Retrieve a payment intent
    pub async fn get_payment_intent(&self, payment_intent_id: &str) -> BillingResult<Payment> {
        let url = format!(
            "{}/v1/payment_intents/{}",
            self.config.api_base, payment_intent_id
        );
        
        let stripe_payment_intent: StripePaymentIntent = self
            .http_client
            .get(&url)
            .bearer_auth(&self.config.stripe_secret_key)
            .send()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| BillingError::PaymentGateway(format!("Failed to parse response: {}", e)))?;
        
        Ok(Payment {
            id: Uuid::new_v4(),
            stripe_payment_intent_id: stripe_payment_intent.id,
            customer_id: Uuid::new_v4(), // Would be mapped from DB
            invoice_id: None,
            amount_cents: stripe_payment_intent.amount,
            currency: stripe_payment_intent.currency,
            status: map_stripe_payment_status(&stripe_payment_intent.status),
            description: stripe_payment_intent.description,
            payment_method: stripe_payment_intent.payment_method,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    /// Retry a failed payment
    pub async fn retry_payment(&self, invoice_id: &str) -> BillingResult<Payment> {
        let invoice = self.pay_invoice(invoice_id).await?;
        
        if let Some(payment_intent_id) = invoice.stripe_invoice_id {
            self.get_payment_intent(payment_intent_id).await
        } else {
            Err(BillingError::PaymentGateway(
                "No payment intent found".to_string(),
            ))
        }
    }
    
    /// Verify a webhook signature
    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: i64,
    ) -> BillingResult<bool> {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;
        
        let webhook_secret = self
            .config
            .webhook_secret
            .as_ref()
            .ok_or_else(|| BillingError::Config("Webhook secret not configured".to_string()))?;
        
        // Check timestamp is within tolerance (5 minutes)
        let now = Utc::now().timestamp();
        if (now - timestamp).abs() > 300 {
            return Err(BillingError::Webhook(
                "Timestamp too old or too new".to_string(),
            ));
        }
        
        // Construct signature string
        let signed_payload = format!("{}.{}", timestamp, hex::encode(payload));
        
        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
            .map_err(|e| BillingError::Webhook(format!("HMAC error: {}", e)))?;
        
        mac.update(signed_payload.as_bytes());
        let expected_signature = hex::encode(mac.finalize().into_bytes());
        
        Ok(signature == expected_signature)
    }
    
    // Helper methods
    
    fn map_stripe_subscription(&self, sub: StripeSubscription) -> BillingResult<Subscription> {
        Ok(Subscription {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(), // Would be mapped from DB
            plan_id: Uuid::new_v4(),     // Would be mapped from DB
            stripe_subscription_id: sub.id,
            status: map_stripe_subscription_status(&sub.status),
            current_period_start: DateTime::from_timestamp(sub.current_period_start, 0)
                .unwrap_or_else(Utc::now),
            current_period_end: DateTime::from_timestamp(sub.current_period_end, 0)
                .unwrap_or_else(Utc::now),
            trial_start: sub.trial_start.map(|ts| {
                DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
            }),
            trial_end: sub.trial_end.map(|ts| {
                DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
            }),
            cancel_at_period_end: sub.cancel_at_period_end,
            cancel_at: sub.cancel_at.map(|ts| {
                DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
            }),
            created_at: DateTime::from_timestamp(sub.created, 0).unwrap_or_else(Utc::now),
            updated_at: Utc::now(),
        })
    }
    
    fn map_stripe_invoice(&self, invoice: StripeInvoice) -> BillingResult<Invoice> {
        Ok(Invoice {
            id: Uuid::new_v4(),
            stripe_invoice_id: Some(invoice.id),
            customer_id: Uuid::new_v4(), // Would be mapped from DB
            subscription_id: invoice.subscription.map(|_| Uuid::new_v4()),
            number: invoice.number,
            status: map_stripe_invoice_status(&invoice.status),
            amount_cents: invoice.total,
            amount_paid_cents: invoice.amount_paid,
            amount_due_cents: invoice.amount_due,
            currency: invoice.currency,
            period_start: DateTime::from_timestamp(invoice.period_start, 0)
                .unwrap_or_else(Utc::now),
            period_end: DateTime::from_timestamp(invoice.period_end, 0)
                .unwrap_or_else(Utc::now),
            due_date: invoice.due_date.map(|ts| {
                DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
            }),
            paid_at: invoice.status_transitions
                .as_ref()
                .and_then(|t| t.paid_at)
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
            description: invoice.description,
            line_items: Vec::new(),
            prorations: Vec::new(),
            pdf_url: invoice.hosted_invoice_url,
            created_at: DateTime::from_timestamp(invoice.created, 0).unwrap_or_else(Utc::now),
            updated_at: Utc::now(),
        })
    }
}

// Data structures for API requests

#[derive(Debug)]
pub struct CustomerData {
    pub email: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProrationBehavior {
    CreateProrations,
    None,
    AlwaysInvoice,
}

impl ProrationBehavior {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateProrations => "create_prorations",
            Self::None => "none",
            Self::AlwaysInvoice => "always_invoice",
        }
    }
}

// Stripe API response structures

#[derive(Debug, Deserialize)]
struct StripeCustomer {
    id: String,
    email: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct StripeSubscription {
    id: String,
    status: String,
    current_period_start: i64,
    current_period_end: i64,
    trial_start: Option<i64>,
    trial_end: Option<i64>,
    cancel_at_period_end: bool,
    cancel_at: Option<i64>,
    created: i64,
}

#[derive(Debug, Deserialize)]
struct StripeInvoice {
    id: String,
    number: String,
    status: String,
    total: i64,
    amount_paid: i64,
    amount_due: i64,
    currency: String,
    period_start: i64,
    period_end: i64,
    due_date: Option<i64>,
    description: Option<String>,
    hosted_invoice_url: Option<String>,
    subscription: Option<String>,
    created: i64,
    status_transitions: Option<StatusTransitions>,
}

#[derive(Debug, Deserialize)]
struct StatusTransitions {
    finalized_at: Option<i64>,
    paid_at: Option<i64>,
    voided_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct StripePaymentIntent {
    id: String,
    status: String,
    amount: i64,
    currency: String,
    description: Option<String>,
    payment_method: Option<String>,
}

// Mapping functions

fn map_stripe_subscription_status(status: &str) -> SubscriptionStatus {
    match status {
        "active" => SubscriptionStatus::Active,
        "trialing" => SubscriptionStatus::Trialing,
        "past_due" => SubscriptionStatus::PastDue,
        "canceled" => SubscriptionStatus::Canceled,
        "unpaid" => SubscriptionStatus::Unpaid,
        "incomplete" => SubscriptionStatus::Incomplete,
        "incomplete_expired" => SubscriptionStatus::IncompleteExpired,
        _ => SubscriptionStatus::Incomplete,
    }
}

fn map_stripe_invoice_status(status: &str) -> InvoiceStatus {
    match status {
        "draft" => InvoiceStatus::Draft,
        "open" => InvoiceStatus::Open,
        "paid" => InvoiceStatus::Paid,
        "void" => InvoiceStatus::Void,
        "uncollectible" => InvoiceStatus::Uncollectible,
        _ => InvoiceStatus::Draft,
    }
}

fn map_stripe_payment_status(status: &str) -> PaymentStatus {
    match status {
        "requires_payment_method" => PaymentStatus::RequiresPaymentMethod,
        "requires_confirmation" => PaymentStatus::RequiresConfirmation,
        "requires_action" => PaymentStatus::RequiresAction,
        "processing" => PaymentStatus::Processing,
        "succeeded" => PaymentStatus::Succeeded,
        "canceled" => PaymentStatus::Canceled,
        "failed" => PaymentStatus::Failed,
        _ => PaymentStatus::RequiresPaymentMethod,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proration_behavior_as_str() {
        assert_eq!(ProrationBehavior::CreateProrations.as_str(), "create_prorations");
        assert_eq!(ProrationBehavior::None.as_str(), "none");
        assert_eq!(ProrationBehavior::AlwaysInvoice.as_str(), "always_invoice");
    }
    
    #[test]
    fn test_map_stripe_subscription_status() {
        assert_eq!(
            map_stripe_subscription_status("active"),
            SubscriptionStatus::Active
        );
        assert_eq!(
            map_stripe_subscription_status("trialing"),
            SubscriptionStatus::Trialing
        );
    }
}

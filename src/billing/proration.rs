//! Proration calculation for plan upgrades, downgrades, and cancellations

use super::models::*;
use super::{BillingError, BillingResult};
use chrono::{Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};

/// Proration calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProrationResult {
    pub credit_amount_cents: i64,
    pub charge_amount_cents: i64,
    pub net_amount_cents: i64,
    pub effective_date: DateTime<Utc>,
    pub prorations: Vec<ProrationItem>,
}

/// Proration calculator
pub struct ProrationCalculator;

impl ProrationCalculator {
    /// Calculate proration for plan upgrade
    pub fn calculate_upgrade_proration(
        &self,
        old_plan: &Plan,
        new_plan: &Plan,
        current_period_start: DateTime<Utc>,
        current_period_end: DateTime<Utc>,
        effective_date: DateTime<Utc>,
    ) -> BillingResult<ProrationResult> {
        if effective_date < current_period_start {
            return Err(BillingError::Proration(
                "Effective date cannot be before current period start".to_string(),
            ));
        }

        if effective_date >= current_period_end {
            return Err(BillingError::Proration(
                "Effective date cannot be after current period end".to_string(),
            ));
        }

        // Calculate remaining days in current period
        let total_days = (current_period_end - current_period_start).num_days();
        let days_used = (effective_date - current_period_start).num_days();
        let days_remaining = total_days - days_used;

        // Calculate daily rates
        let old_daily_rate = old_plan.amount_cents as f64 / total_days as f64;
        let new_daily_rate = new_plan.amount_cents as f64 / total_days as f64;

        // Calculate credit for unused portion of old plan
        let unused_days = days_remaining as f64;
        let credit_amount = (old_daily_rate * unused_days).round() as i64;

        // Calculate charge for remaining period with new plan
        let charge_amount = (new_daily_rate * unused_days).round() as i64;

        // Net amount
        let net_amount = charge_amount - credit_amount;

        // If net is negative, we owe the customer a refund
        // If net is positive, the customer owes us the difference

        let prorations = vec![
            ProrationItem {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "Credit for unused portion of {} plan ({} days)",
                    old_plan.name, days_remaining
                ),
                amount_cents: -credit_amount,
                period_start: effective_date,
                period_end: current_period_end,
                proration_type: ProrationType::Credit,
            },
            ProrationItem {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "Upgrade to {} plan for remaining {} days",
                    new_plan.name, days_remaining
                ),
                amount_cents: charge_amount,
                period_start: effective_date,
                period_end: current_period_end,
                proration_type: ProrationType::Upgrade,
            },
        ];

        Ok(ProrationResult {
            credit_amount_cents: credit_amount,
            charge_amount_cents: charge_amount,
            net_amount_cents: net_amount,
            effective_date,
            prorations,
        })
    }

    /// Calculate proration for plan downgrade
    pub fn calculate_downgrade_proration(
        &self,
        old_plan: &Plan,
        new_plan: &Plan,
        current_period_start: DateTime<Utc>,
        current_period_end: DateTime<Utc>,
        effective_date: DateTime<Utc>,
    ) -> BillingResult<ProrationResult> {
        // Downgrades typically take effect at the next billing cycle
        // But we can prorate if requested

        if effective_date < current_period_start {
            return Err(BillingError::Proration(
                "Effective date cannot be before current period start".to_string(),
            ));
        }

        let total_days = (current_period_end - current_period_start).num_days();
        let days_used = (effective_date - current_period_start).num_days();
        let days_remaining = total_days - days_used;

        let old_daily_rate = old_plan.amount_cents as f64 / total_days as f64;
        let new_daily_rate = new_plan.amount_cents as f64 / total_days as f64;

        // Credit for unused portion of old plan
        let unused_days = days_remaining as f64;
        let credit_amount = (old_daily_rate * unused_days).round() as i64;

        // Charge for remaining period with new (cheaper) plan
        let charge_amount = (new_daily_rate * unused_days).round() as i64;

        // Net amount (customer gets credit)
        let net_amount = charge_amount - credit_amount;

        let prorations = vec![
            ProrationItem {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "Credit for unused portion of {} plan ({} days)",
                    old_plan.name, days_remaining
                ),
                amount_cents: -credit_amount,
                period_start: effective_date,
                period_end: current_period_end,
                proration_type: ProrationType::Credit,
            },
            ProrationItem {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "Downgrade to {} plan for remaining {} days",
                    new_plan.name, days_remaining
                ),
                amount_cents: charge_amount,
                period_start: effective_date,
                period_end: current_period_end,
                proration_type: ProrationType::Downgrade,
            },
        ];

        Ok(ProrationResult {
            credit_amount_cents: credit_amount,
            charge_amount_cents: charge_amount,
            net_amount_cents: net_amount,
            effective_date,
            prorations,
        })
    }

    /// Calculate proration for subscription cancellation
    pub fn calculate_cancellation_proration(
        &self,
        plan: &Plan,
        current_period_start: DateTime<Utc>,
        current_period_end: DateTime<Utc>,
        cancellation_date: DateTime<Utc>,
        refund_policy: RefundPolicy,
    ) -> BillingResult<ProrationResult> {
        let total_days = (current_period_end - current_period_start).num_days();
        let days_used = (cancellation_date - current_period_start).num_days();
        let days_remaining = total_days - days_used;

        let daily_rate = plan.amount_cents as f64 / total_days as f64;
        let refund_amount = match refund_policy {
            RefundPolicy::FullRefund => plan.amount_cents,
            RefundPolicy::ProratedRefund => {
                // Full refund for remaining days
                ((daily_rate * days_remaining as f64).round() as i64).max(0)
            }
            RefundPolicy::NoRefund => 0,
            RefundPolicy::PartialRefund(percent) => {
                let max_refund = (daily_rate * days_remaining as f64).round() as i64;
                (max_refund * percent / 100).max(0)
            }
        };

        let prorations = if refund_amount > 0 {
            vec![ProrationItem {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "Refund for {} plan - {} unused days ({} policy)",
                    plan.name, days_remaining, refund_policy
                ),
                amount_cents: refund_amount,
                period_start: cancellation_date,
                period_end: current_period_end,
                proration_type: ProrationType::Cancellation,
            }]
        } else {
            vec![]
        };

        Ok(ProrationResult {
            credit_amount_cents: refund_amount,
            charge_amount_cents: 0,
            net_amount_cents: -refund_amount,
            effective_date: cancellation_date,
            prorations,
        })
    }

    /// Calculate proration for trial period ending
    pub fn calculate_trial_end_proration(
        &self,
        plan: &Plan,
        trial_end_date: DateTime<Utc>,
    ) -> BillingResult<ProrationResult> {
        // When trial ends, customer is charged the full plan amount
        // starting from trial_end_date

        let prorations = vec![ProrationItem {
            id: uuid::Uuid::new_v4(),
            description: format!("Trial ended - subscription to {} plan begins", plan.name),
            amount_cents: plan.amount_cents,
            period_start: trial_end_date,
            period_end: trial_end_date
                + Duration::days(Self::get_period_days(plan.interval) as i64),
            proration_type: ProrationType::Upgrade,
        }];

        Ok(ProrationResult {
            credit_amount_cents: 0,
            charge_amount_cents: plan.amount_cents,
            net_amount_cents: plan.amount_cents,
            effective_date: trial_end_date,
            prorations,
        })
    }

    /// Calculate proration for adding a trial extension
    pub fn calculate_trial_extension_proration(
        &self,
        plan: &Plan,
        extension_days: u32,
    ) -> BillingResult<ProrationResult> {
        // Trial extensions are typically free, but can be logged

        let prorations = vec![ProrationItem {
            id: uuid::Uuid::new_v4(),
            description: format!(
                "Trial extension: +{} days for {} plan",
                extension_days, plan.name
            ),
            amount_cents: 0,
            period_start: Utc::now(),
            period_end: Utc::now() + Duration::days(extension_days as i64),
            proration_type: ProrationType::Credit,
        }];

        Ok(ProrationResult {
            credit_amount_cents: 0,
            charge_amount_cents: 0,
            net_amount_cents: 0,
            effective_date: Utc::now(),
            prorations,
        })
    }

    // Helper methods

    fn get_period_days(interval: BillingInterval) -> u32 {
        match interval {
            BillingInterval::Monthly => 30,
            BillingInterval::Yearly => 365,
        }
    }
}

/// Refund policy for cancellations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundPolicy {
    FullRefund,
    ProratedRefund,
    PartialRefund(u8), // percentage
    NoRefund,
}

impl std::fmt::Display for RefundPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullRefund => write!(f, "Full Refund"),
            Self::ProratedRefund => write!(f, "Prorated Refund"),
            Self::PartialRefund(p) => write!(f, "Partial Refund ({}%)", p),
            Self::NoRefund => write!(f, "No Refund"),
        }
    }
}

/// Proration calculator builder for common scenarios
pub struct ProrationBuilder {
    old_plan: Option<Plan>,
    new_plan: Option<Plan>,
    current_period_start: Option<DateTime<Utc>>,
    current_period_end: Option<DateTime<Utc>>,
    effective_date: Option<DateTime<Utc>>,
    refund_policy: RefundPolicy,
}

impl Default for ProrationBuilder {
    fn default() -> Self {
        Self {
            old_plan: None,
            new_plan: None,
            current_period_start: None,
            current_period_end: None,
            effective_date: None,
            refund_policy: RefundPolicy::ProratedRefund,
        }
    }
}

impl ProrationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_old_plan(mut self, plan: Plan) -> Self {
        self.old_plan = Some(plan);
        self
    }

    pub fn with_new_plan(mut self, plan: Plan) -> Self {
        self.new_plan = Some(plan);
        self
    }

    pub fn with_period(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.current_period_start = Some(start);
        self.current_period_end = Some(end);
        self
    }

    pub fn with_effective_date(mut self, date: DateTime<Utc>) -> Self {
        self.effective_date = Some(date);
        self
    }

    pub fn with_refund_policy(mut self, policy: RefundPolicy) -> Self {
        self.refund_policy = policy;
        self
    }

    pub fn calculate_upgrade(self) -> BillingResult<ProrationResult> {
        let calculator = ProrationCalculator;
        calculator.calculate_upgrade_proration(
            self.old_plan.as_ref().ok_or_else(|| BillingError::Proration("Old plan required".to_string()))?,
            self.new_plan.as_ref().ok_or_else(|| BillingError::Proration("New plan required".to_string()))?,
            self.current_period_start.ok_or_else(|| BillingError::Proration("Period start required".to_string()))?,
            self.current_period_end.ok_or_else(|| BillingError::Proration("Period end required".to_string()))?,
            self.effective_date.ok_or_else(|| BillingError::Proration("Effective date required".to_string()))?,
        )
    }

    pub fn calculate_downgrade(self) -> BillingResult<ProrationResult> {
        let calculator = ProrationCalculator;
        calculator.calculate_downgrade_proration(
            self.old_plan.as_ref().ok_or_else(|| BillingError::Proration("Old plan required".to_string()))?,
            self.new_plan.as_ref().ok_or_else(|| BillingError::Proration("New plan required".to_string()))?,
            self.current_period_start.ok_or_else(|| BillingError::Proration("Period start required".to_string()))?,
            self.current_period_end.ok_or_else(|| BillingError::Proration("Period end required".to_string()))?,
            self.effective_date.ok_or_else(|| BillingError::Proration("Effective date required".to_string()))?,
        )
    }

    pub fn calculate_cancellation(self) -> BillingResult<ProrationResult> {
        let calculator = ProrationCalculator;
        calculator.calculate_cancellation_proration(
            self.old_plan.as_ref().ok_or_else(|| BillingError::Proration("Old plan required".to_string()))?,
            self.current_period_start.ok_or_else(|| BillingError::Proration("Period start required".to_string()))?,
            self.current_period_end.ok_or_else(|| BillingError::Proration("Period end required".to_string()))?,
            self.effective_date.ok_or_else(|| BillingError::Proration("Effective date required".to_string()))?,
            self.refund_policy,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_proration() {
        let calculator = ProrationCalculator;
        let now = Utc::now();
        let period_start = now - Duration::days(10);
        let period_end = now + Duration::days(20);

        let old_plan = Plan {
            id: uuid::Uuid::new_v4(),
            stripe_price_id: "price_old".to_string(),
            name: "Basic".to_string(),
            description: None,
            amount_cents: 1000, // $10.00
            currency: "USD".to_string(),
            interval: BillingInterval::Monthly,
            trial_days: None,
            metadata: serde_json::json!({}),
        };

        let new_plan = Plan {
            id: uuid::Uuid::new_v4(),
            stripe_price_id: "price_new".to_string(),
            name: "Premium".to_string(),
            description: None,
            amount_cents: 2500, // $25.00
            currency: "USD".to_string(),
            interval: BillingInterval::Monthly,
            trial_days: None,
            metadata: serde_json::json!({}),
        };

        let result = calculator
            .calculate_upgrade_proration(&old_plan, &new_plan, period_start, period_end, now)
            .unwrap();

        // Customer should be charged for the difference
        assert!(result.net_amount_cents > 0);
        assert_eq!(result.prorations.len(), 2);
    }

    #[test]
    fn test_cancellation_proration() {
        let calculator = ProrationCalculator;
        let now = Utc::now();
        let period_start = now - Duration::days(10);
        let period_end = now + Duration::days(20);

        let plan = Plan {
            id: uuid::Uuid::new_v4(),
            stripe_price_id: "price".to_string(),
            name: "Premium".to_string(),
            description: None,
            amount_cents: 2500, // $25.00
            currency: "USD".to_string(),
            interval: BillingInterval::Monthly,
            trial_days: None,
            metadata: serde_json::json!({}),
        };

        let result = calculator
            .calculate_cancellation_proration(
                &plan,
                period_start,
                period_end,
                now,
                RefundPolicy::ProratedRefund,
            )
            .unwrap();

        // Customer should get a refund
        assert!(result.net_amount_cents < 0);
        assert!(result.prorations.len() == 1);
    }
}

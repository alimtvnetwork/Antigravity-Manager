//! Comprehensive Account Readiness Evaluator
//! Evaluates token expiration, cluster lease status, 4-hour quota sufficiency,
//! and cooldown state to generate an authoritative readiness determination.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountReadinessReport {
    pub account_id: String,
    pub is_ready: bool,
    pub token_valid: bool,
    pub lease_available: bool,
    pub quota_sufficient: bool,
    pub in_cooldown: bool,
    pub primary_gate_passed: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReadinessEvaluator;

impl ReadinessEvaluator {
    /// Performs comprehensive evaluation of candidate readiness.
    pub fn evaluate(
        account_id: impl Into<String>,
        token_expires_at_epoch_sec: i64,
        now_epoch_sec: i64,
        is_leased_by_other_node: bool,
        four_hour_quota_pct: f64,
        is_cooldown: bool,
    ) -> AccountReadinessReport {
        let account_id_str = account_id.into();
        let token_valid = (token_expires_at_epoch_sec - now_epoch_sec) > 60; // At least 60s buffer
        let lease_available = !is_leased_by_other_node;
        let quota_sufficient = four_hour_quota_pct > 0.0;
        let primary_gate_passed = four_hour_quota_pct >= 99.5;

        let mut rejection_reason = None;
        if is_cooldown {
            rejection_reason = Some("Account currently in cooldown throttle".to_string());
        } else if !token_valid {
            rejection_reason = Some("OAuth access token expired or nearing expiration".to_string());
        } else if !lease_available {
            rejection_reason = Some("Account actively leased by another cluster machine".to_string());
        } else if !quota_sufficient {
            rejection_reason = Some("Account 4-hour quota depleted (0.0% remaining)".to_string());
        }

        let is_ready = token_valid && lease_available && quota_sufficient && !is_cooldown;

        AccountReadinessReport {
            account_id: account_id_str,
            is_ready,
            token_valid,
            lease_available,
            quota_sufficient,
            in_cooldown,
            primary_gate_passed,
            rejection_reason,
        }
    }
}

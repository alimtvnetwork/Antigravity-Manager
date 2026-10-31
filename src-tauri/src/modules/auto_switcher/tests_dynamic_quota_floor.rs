//! Unit tests for Dynamic Quota Safety Floor Guard (Imp-563).

use super::dynamic_quota_floor::{DynamicQuotaFloorGuard, ModelConsumptionTier};

#[test]
fn test_model_consumption_tier_classification() {
    assert_eq!(
        ModelConsumptionTier::from_model_name("gemini-2.0-flash-lite"),
        ModelConsumptionTier::Lightweight
    );
    assert_eq!(
        ModelConsumptionTier::from_model_name("gemini-2.5-flash"),
        ModelConsumptionTier::Standard
    );
    assert_eq!(
        ModelConsumptionTier::from_model_name("claude-3-5-sonnet-20241022"),
        ModelConsumptionTier::Heavyweight
    );
    assert_eq!(
        ModelConsumptionTier::from_model_name("gemini-2.0-flash-thinking-exp"),
        ModelConsumptionTier::Reasoning
    );
}

#[test]
fn test_minimum_safe_headroom_percent() {
    assert_eq!(
        ModelConsumptionTier::Lightweight.minimum_safe_headroom_percent(),
        3.0
    );
    assert_eq!(
        ModelConsumptionTier::Standard.minimum_safe_headroom_percent(),
        5.0
    );
    assert_eq!(
        ModelConsumptionTier::Heavyweight.minimum_safe_headroom_percent(),
        8.0
    );
    assert_eq!(
        ModelConsumptionTier::Reasoning.minimum_safe_headroom_percent(),
        12.0
    );
}

#[test]
fn test_evaluate_quota_floor_sufficient() {
    let eval = DynamicQuotaFloorGuard::evaluate("claude-3-5-sonnet", 15.0);
    assert_eq!(eval.model_tier, ModelConsumptionTier::Heavyweight);
    assert_eq!(eval.minimum_required_percent, 8.0);
    assert!(eval.is_sufficient);
    assert_eq!(eval.headroom_margin, 7.0);
}

#[test]
fn test_evaluate_quota_floor_insufficient() {
    let eval = DynamicQuotaFloorGuard::evaluate("gemini-2.0-flash-thinking", 10.0);
    assert_eq!(eval.model_tier, ModelConsumptionTier::Reasoning);
    assert_eq!(eval.minimum_required_percent, 12.0);
    assert!(!eval.is_sufficient);
    assert_eq!(eval.headroom_margin, -2.0);
}

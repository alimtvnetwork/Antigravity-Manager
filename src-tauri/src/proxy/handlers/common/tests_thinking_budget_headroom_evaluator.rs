//! Unit tests for Thinking Budget Headroom & Deficit Evaluator (`Imp-653`).

use super::thinking_budget_headroom_evaluator::{
    evaluate_thinking_budget_headroom, evaluate_thinking_budget_headroom_default,
    HeadroomAdjustmentAction, HeadroomEvaluationReport, HeadroomStatus,
    CONTEXT_TOKEN_CEILING_DEFAULT, MIN_HEADROOM_TOKENS_DEFAULT,
};

#[test]
fn test_thinking_disabled_produces_thinking_disabled_and_valid_report() {
    let report = evaluate_thinking_budget_headroom(
        Some(2048),
        None,
        MIN_HEADROOM_TOKENS_DEFAULT,
        CONTEXT_TOKEN_CEILING_DEFAULT,
    );

    assert!(report.is_valid());
    assert!(!report.has_thinking());
    assert!(!report.is_adjusted());
    assert_eq!(report.status(), &HeadroomStatus::ThinkingDisabled);
    assert_eq!(report.action(), &HeadroomAdjustmentAction::NoAdjustmentNeeded);
    assert_eq!(report.effective_max_output_tokens, 2048);
    assert_eq!(report.effective_thinking_budget, None);

    // Also test Some(0) budget
    let report_zero = evaluate_thinking_budget_headroom(
        None,
        Some(0),
        MIN_HEADROOM_TOKENS_DEFAULT,
        CONTEXT_TOKEN_CEILING_DEFAULT,
    );
    assert!(report_zero.is_valid());
    assert!(!report_zero.has_thinking());
    assert_eq!(report_zero.status(), &HeadroomStatus::ThinkingDisabled);
    assert_eq!(report_zero.effective_max_output_tokens, 4096);
}

#[test]
fn test_sufficient_headroom_produces_sufficient_headroom_and_no_adjustment() {
    let max_output = Some(8192);
    let thinking_budget = Some(2048);

    let report = evaluate_thinking_budget_headroom(
        max_output,
        thinking_budget,
        MIN_HEADROOM_TOKENS_DEFAULT,
        CONTEXT_TOKEN_CEILING_DEFAULT,
    );

    assert!(report.is_valid());
    assert!(report.has_thinking());
    assert!(!report.is_adjusted());
    assert_eq!(report.action(), &HeadroomAdjustmentAction::NoAdjustmentNeeded);
    assert_eq!(report.effective_max_output_tokens, 8192);
    assert_eq!(report.effective_thinking_budget, Some(2048));

    match report.status() {
        HeadroomStatus::SufficientHeadroom { headroom_tokens } => {
            assert_eq!(*headroom_tokens, 8192 - 2048);
        }
        other => panic!("Expected SufficientHeadroom, got: {:?}", other),
    }
}

#[test]
fn test_insufficient_max_output_expanded_to_budget_plus_min_headroom() {
    let max_output = Some(2048);
    let thinking_budget = Some(2048);

    let report = evaluate_thinking_budget_headroom(
        max_output,
        thinking_budget,
        MIN_HEADROOM_TOKENS_DEFAULT,
        CONTEXT_TOKEN_CEILING_DEFAULT,
    );

    assert!(report.is_valid());
    assert!(report.has_thinking());
    assert!(report.is_adjusted());
    assert_eq!(report.effective_max_output_tokens, 2048 + 1024);
    assert_eq!(report.effective_thinking_budget, Some(2048));

    match report.status() {
        HeadroomStatus::InsufficientHeadroom {
            max_output: orig,
            thinking_budget: b,
            deficit,
        } => {
            assert_eq!(*orig, 2048);
            assert_eq!(*b, 2048);
            assert_eq!(*deficit, 1024);
        }
        other => panic!("Expected InsufficientHeadroom, got: {:?}", other),
    }

    match report.action() {
        HeadroomAdjustmentAction::ExpandedMaxOutputTokens {
            original_max,
            adjusted_max,
        } => {
            assert_eq!(*original_max, 2048);
            assert_eq!(*adjusted_max, 3072);
        }
        other => panic!("Expected ExpandedMaxOutputTokens, got: {:?}", other),
    }
}

#[test]
fn test_unspecified_max_output_defaults_to_sufficient_headroom() {
    let report = evaluate_thinking_budget_headroom_default(None, Some(4096));

    assert!(report.is_valid());
    assert!(report.has_thinking());
    assert!(!report.is_adjusted());
    assert_eq!(report.effective_max_output_tokens, 4096 + 1024);
    assert_eq!(report.effective_thinking_budget, Some(4096));
    assert_eq!(report.action(), &HeadroomAdjustmentAction::NoAdjustmentNeeded);
}

#[test]
fn test_exceeding_ceiling_clamps_thinking_budget_when_feasible() {
    let max_output = Some(65000);
    let thinking_budget = Some(65000);

    let report = evaluate_thinking_budget_headroom(
        max_output,
        thinking_budget,
        1024,
        65536,
    );

    assert!(report.is_valid());
    assert!(report.has_thinking());
    assert!(report.is_adjusted());
    assert_eq!(report.effective_max_output_tokens, 65536);

    let expected_clamped = 65536 - 1024;
    assert_eq!(report.effective_thinking_budget, Some(expected_clamped));

    match report.action() {
        HeadroomAdjustmentAction::ClampedThinkingBudget {
            original_budget,
            clamped_budget,
        } => {
            assert_eq!(*original_budget, 65000);
            assert_eq!(*clamped_budget, expected_clamped);
        }
        other => panic!("Expected ClampedThinkingBudget, got: {:?}", other),
    }
}

#[test]
fn test_extreme_over_ceiling_returns_rejection_and_invalid() {
    let max_output = Some(2000);
    let thinking_budget = Some(2000);
    let ceiling = 1200;
    let min_headroom = 1024;

    let report = evaluate_thinking_budget_headroom(
        max_output,
        thinking_budget,
        min_headroom,
        ceiling,
    );

    assert!(!report.is_valid());
    assert!(report.is_adjusted());

    match report.action() {
        HeadroomAdjustmentAction::RejectedDueToExceedingContextCeiling {
            requested_total,
            ceiling: c,
        } => {
            assert_eq!(*requested_total, 2000 + 1024);
            assert_eq!(*c, 1200);
        }
        other => panic!("Expected RejectedDueToExceedingContextCeiling, got: {:?}", other),
    }
}

#[test]
fn test_serde_roundtrip_sufficient() {
    let report = HeadroomEvaluationReport {
        status: HeadroomStatus::SufficientHeadroom {
            headroom_tokens: 4096,
        },
        action: HeadroomAdjustmentAction::NoAdjustmentNeeded,
        effective_max_output_tokens: 8192,
        effective_thinking_budget: Some(4096),
        is_valid: true,
    };

    let serialized = serde_json::to_string(&report).expect("Serialization must succeed");
    let deserialized: HeadroomEvaluationReport =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");

    assert_eq!(report, deserialized);
}

#[test]
fn test_serde_roundtrip_adjusted_and_rejected() {
    let report_expanded = HeadroomEvaluationReport {
        status: HeadroomStatus::InsufficientHeadroom {
            max_output: 1024,
            thinking_budget: 1024,
            deficit: 1024,
        },
        action: HeadroomAdjustmentAction::ExpandedMaxOutputTokens {
            original_max: 1024,
            adjusted_max: 2048,
        },
        effective_max_output_tokens: 2048,
        effective_thinking_budget: Some(1024),
        is_valid: true,
    };

    let serialized = serde_json::to_string(&report_expanded).expect("Serialization must succeed");
    let deserialized: HeadroomEvaluationReport =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");
    assert_eq!(report_expanded, deserialized);

    let report_rejected = HeadroomEvaluationReport {
        status: HeadroomStatus::InsufficientHeadroom {
            max_output: 5000,
            thinking_budget: 5000,
            deficit: 1024,
        },
        action: HeadroomAdjustmentAction::RejectedDueToExceedingContextCeiling {
            requested_total: 6024,
            ceiling: 1000,
        },
        effective_max_output_tokens: 5000,
        effective_thinking_budget: Some(5000),
        is_valid: false,
    };

    let serialized_rej =
        serde_json::to_string(&report_rejected).expect("Serialization must succeed");
    let deserialized_rej: HeadroomEvaluationReport =
        serde_json::from_str(&serialized_rej).expect("Deserialization must succeed");
    assert_eq!(report_rejected, deserialized_rej);
}

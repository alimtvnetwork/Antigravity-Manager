//! Unit tests for Account Switch Audit Emitter (`Imp-623`).
//!
//! Tests structured JSON telemetry, trigger reason representations, candidate
//! scores ranking breakdown, builder mechanics, error propagation, and JSON parsing.

use super::account_switch_audit_emitter::{
    build_switch_audit_event, clear_audit_events, emit_account_switch_audit,
    export_buffered_events_json, format_audit_event_json, get_recent_audit_events,
    parse_audit_event_json, rank_candidate_scores, total_emitted_audit_events_count,
    AccountSwitchAuditEvent, AccountSwitchAuditEmitter, CandidateAuditScore, SwitchTriggerReason,
};

#[test]
fn test_switch_trigger_reason_variants_and_formatting() {
    let variants = [
        (SwitchTriggerReason::RateLimit429, "RATE_LIMIT_429"),
        (
            SwitchTriggerReason::PreemptiveExhaustionVelocity,
            "PREEMPTIVE_EXHAUSTION_VELOCITY",
        ),
        (SwitchTriggerReason::ManualOverride, "MANUAL_OVERRIDE"),
        (SwitchTriggerReason::HealthDecay, "HEALTH_DECAY"),
        (SwitchTriggerReason::CooldownExpired, "COOLDOWN_EXPIRED"),
        (SwitchTriggerReason::InitialSelection, "INITIAL_SELECTION"),
    ];

    for (variant, expected_str) in variants {
        assert_eq!(variant.as_str(), expected_str);
        assert_eq!(format!("{variant}"), expected_str);

        // Test parsing by uppercase identifier
        assert_eq!(SwitchTriggerReason::from_str_name(expected_str), Some(variant));
    }

    // Test parsing by Rust variant name
    assert_eq!(
        SwitchTriggerReason::from_str_name("RateLimit429"),
        Some(SwitchTriggerReason::RateLimit429)
    );
    assert_eq!(
        SwitchTriggerReason::from_str_name("PreemptiveExhaustionVelocity"),
        Some(SwitchTriggerReason::PreemptiveExhaustionVelocity)
    );
    assert_eq!(
        SwitchTriggerReason::from_str_name("ManualOverride"),
        Some(SwitchTriggerReason::ManualOverride)
    );
    assert_eq!(
        SwitchTriggerReason::from_str_name("HealthDecay"),
        Some(SwitchTriggerReason::HealthDecay)
    );
    assert_eq!(
        SwitchTriggerReason::from_str_name("CooldownExpired"),
        Some(SwitchTriggerReason::CooldownExpired)
    );
    assert_eq!(
        SwitchTriggerReason::from_str_name("InitialSelection"),
        Some(SwitchTriggerReason::InitialSelection)
    );

    // Test invalid variant name returns None
    assert_eq!(SwitchTriggerReason::from_str_name("UnknownReason"), None);
}

#[test]
fn test_candidate_audit_score_creation_and_fields() {
    let candidate = CandidateAuditScore::new(
        "acc_beta_001",
        875.5,
        0.85,
        15.0,
        5.0,
        false,
    );

    assert_eq!(candidate.candidate_account_id, "acc_beta_001");
    assert_eq!(candidate.final_score, 875.5);
    assert_eq!(candidate.quota_headroom, 0.85);
    assert_eq!(candidate.burn_rate_velocity_penalty, 15.0);
    assert_eq!(candidate.throttle_penalty, 5.0);
    assert!(!candidate.cooldown_status);
}

#[test]
fn test_candidate_scores_ranking_breakdown() {
    let c1 = CandidateAuditScore::new("acc_1", 520.0, 0.40, 20.0, 10.0, false);
    let c2 = CandidateAuditScore::new("acc_2", 950.0, 0.95, 0.0, 0.0, false);
    let c3 = CandidateAuditScore::new("acc_3", 810.0, 0.70, 5.0, 0.0, true);
    let c4 = CandidateAuditScore::new("acc_4", 310.0, 0.15, 45.0, 30.0, true);

    let candidates = vec![c1, c2, c3, c4];
    let ranked = rank_candidate_scores(&candidates);

    assert_eq!(ranked.len(), 4);
    // Highest final score first
    assert_eq!(ranked[0].candidate_account_id, "acc_2");
    assert_eq!(ranked[0].final_score, 950.0);
    assert_eq!(ranked[1].candidate_account_id, "acc_3");
    assert_eq!(ranked[1].final_score, 810.0);
    assert_eq!(ranked[2].candidate_account_id, "acc_1");
    assert_eq!(ranked[2].final_score, 520.0);
    assert_eq!(ranked[3].candidate_account_id, "acc_4");
    assert_eq!(ranked[3].final_score, 310.0);

    // Also verify via AccountSwitchAuditEvent::ranked_candidate_scores
    let event = AccountSwitchAuditEvent::new(
        Some("acc_1".to_string()),
        "acc_2",
        SwitchTriggerReason::PreemptiveExhaustionVelocity,
    )
    .with_candidate_scores(candidates);

    let event_ranked = event.ranked_candidate_scores();
    assert_eq!(event_ranked[0].candidate_account_id, "acc_2");
    assert_eq!(event_ranked[3].candidate_account_id, "acc_4");

    // Target candidate lookup
    let target_score = event.target_candidate_score().expect("target score present");
    assert_eq!(target_score.candidate_account_id, "acc_2");
    assert_eq!(target_score.final_score, 950.0);
}

#[test]
fn test_build_switch_audit_event_builder() {
    let candidate = CandidateAuditScore::new("acc_target", 920.0, 0.90, 5.0, 0.0, false);

    let event = build_switch_audit_event(
        Some("acc_source_429".to_string()),
        "acc_target",
        SwitchTriggerReason::RateLimit429,
        vec![candidate],
        Some(1800),
        Some(300),
        true,
        None,
    );

    // Verify UUID format
    assert!(!event.event_id.is_empty());
    assert!(uuid::Uuid::parse_str(&event.event_id).is_ok());

    assert!(event.timestamp_epoch_ms > 0);
    assert_eq!(event.source_account_id, Some("acc_source_429".to_string()));
    assert_eq!(event.target_account_id, "acc_target");
    assert_eq!(event.trigger_reason, SwitchTriggerReason::RateLimit429);
    assert_eq!(event.candidate_scores.len(), 1);
    assert_eq!(event.forecast_runway_seconds, Some(1800));
    assert_eq!(event.cooldown_seconds, Some(300));
    assert!(event.success);
    assert_eq!(event.error_message, None);
}

#[test]
fn test_serialization_and_json_parsing_roundtrip() {
    let c1 = CandidateAuditScore::new("acc_winner", 990.0, 0.99, 0.0, 0.0, false);
    let c2 = CandidateAuditScore::new("acc_runner_up", 850.0, 0.80, 10.0, 5.0, false);

    let original_event = build_switch_audit_event(
        Some("acc_decayed".to_string()),
        "acc_winner",
        SwitchTriggerReason::HealthDecay,
        vec![c1, c2],
        Some(7200),
        Some(120),
        true,
        None,
    );

    let json = format_audit_event_json(&original_event).expect("format to JSON");
    assert!(json.contains("\"event_id\":"));
    assert!(json.contains("\"target_account_id\": \"acc_winner\""));
    assert!(json.contains("\"trigger_reason\": \"HealthDecay\""));
    assert!(json.contains("\"candidate_scores\":"));
    assert!(json.contains("\"forecast_runway_seconds\": 7200"));
    assert!(json.contains("\"cooldown_seconds\": 120"));
    assert!(json.contains("\"success\": true"));

    // Parse back
    let parsed_event = parse_audit_event_json(&json).expect("parse JSON");
    assert_eq!(original_event.event_id, parsed_event.event_id);
    assert_eq!(
        original_event.timestamp_epoch_ms,
        parsed_event.timestamp_epoch_ms
    );
    assert_eq!(
        original_event.source_account_id,
        parsed_event.source_account_id
    );
    assert_eq!(
        original_event.target_account_id,
        parsed_event.target_account_id
    );
    assert_eq!(original_event.trigger_reason, parsed_event.trigger_reason);
    assert_eq!(
        original_event.candidate_scores,
        parsed_event.candidate_scores
    );
    assert_eq!(
        original_event.forecast_runway_seconds,
        parsed_event.forecast_runway_seconds
    );
    assert_eq!(
        original_event.cooldown_seconds,
        parsed_event.cooldown_seconds
    );
    assert_eq!(original_event.success, parsed_event.success);
    assert_eq!(original_event.error_message, parsed_event.error_message);
}

#[test]
fn test_parse_audit_event_json_error_propagation() {
    let invalid_json = "{ \"event_id\": \"not_closed_json\" ";
    let err = parse_audit_event_json(invalid_json).expect_err("should return AppError on malformed JSON");

    match err {
        crate::error::AppError::Config(msg) => {
            assert!(msg.contains("Failed to parse AccountSwitchAuditEvent from JSON"));
        }
        other => panic!("Expected AppError::Config, got {other:?}"),
    }
}

#[test]
fn test_emit_account_switch_audit_lifecycle_and_ring_buffer() {
    clear_audit_events();
    assert_eq!(total_emitted_audit_events_count(), 0);

    let success_event = build_switch_audit_event(
        None,
        "acc_first_init",
        SwitchTriggerReason::InitialSelection,
        Vec::new(),
        None,
        None,
        true,
        None,
    );

    let failure_event = build_switch_audit_event(
        Some("acc_old".to_string()),
        "acc_failed_target",
        SwitchTriggerReason::ManualOverride,
        Vec::new(),
        None,
        None,
        false,
        Some("Connection refused on target auth token endpoint".to_string()),
    );

    emit_account_switch_audit(&success_event).expect("emit success event");
    emit_account_switch_audit(&failure_event).expect("emit failure event");

    assert_eq!(total_emitted_audit_events_count(), 2);

    let recents = get_recent_audit_events(5);
    assert_eq!(recents.len(), 2);

    // Newest event first
    assert_eq!(recents[0].target_account_id, "acc_failed_target");
    assert!(!recents[0].success);
    assert_eq!(
        recents[0].error_message.as_deref(),
        Some("Connection refused on target auth token endpoint")
    );

    assert_eq!(recents[1].target_account_id, "acc_first_init");
    assert!(recents[1].success);

    // Export JSON array
    let exported = export_buffered_events_json().expect("export JSON array");
    assert!(exported.contains("acc_failed_target"));
    assert!(exported.contains("acc_first_init"));

    clear_audit_events();
    assert_eq!(total_emitted_audit_events_count(), 0);
    assert!(get_recent_audit_events(10).is_empty());
}

#[test]
fn test_static_emitter_wrapper_facade() {
    AccountSwitchAuditEmitter::clear();
    assert_eq!(AccountSwitchAuditEmitter::total_emitted_count(), 0);

    let event = build_switch_audit_event(
        Some("acc_cooldown".to_string()),
        "acc_recovered",
        SwitchTriggerReason::CooldownExpired,
        Vec::new(),
        Some(3600),
        None,
        true,
        None,
    );

    let json = AccountSwitchAuditEmitter::emit(&event).expect("static emitter emit");
    assert!(json.contains("\"trigger_reason\": \"CooldownExpired\""));
    assert_eq!(AccountSwitchAuditEmitter::total_emitted_count(), 1);

    let events = AccountSwitchAuditEmitter::recent_events(1);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].target_account_id, "acc_recovered");

    AccountSwitchAuditEmitter::clear();
    assert_eq!(AccountSwitchAuditEmitter::total_emitted_count(), 0);
}

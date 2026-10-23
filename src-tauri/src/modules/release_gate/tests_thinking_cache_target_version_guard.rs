//! Unit tests for Thinking Cache Invalidation Target Version Guard (Imp-627).

use crate::modules::release_gate::thinking_cache_target_version_guard::{
    evaluate_thinking_cache_configuration, ReleaseType, ThinkingCacheConfig,
    ThinkingCacheGuardViolation, ThinkingCacheReport,
};

#[test]
fn test_compliant_routine_release_passes_clean() {
    let config = ThinkingCacheConfig::routine("4.8.2");
    let report = evaluate_thinking_cache_configuration(&config, &ReleaseType::RoutineRelease);

    assert!(
        report.is_compliant,
        "Routine release with prompt disabled must be compliant"
    );
    assert!(report.is_clean());
    assert!(report.violations.is_empty());
    assert!(report.guidance.contains("compliant with AGENTS.md"));
}

#[test]
fn test_routine_release_erroneously_enabling_prompt_is_flagged() {
    let config = ThinkingCacheConfig::refactor("4.8.2", "4.8.2");
    let report = evaluate_thinking_cache_configuration(&config, &ReleaseType::RoutineRelease);

    assert!(
        !report.is_compliant,
        "Routine release with prompt enabled must be flagged"
    );
    assert!(!report.is_clean());
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ThinkingCacheGuardViolation::RoutineReleaseErroneouslyPrompts { target_version } => {
            assert_eq!(target_version.as_deref(), Some("4.8.2"));
        }
        other => panic!("Unexpected violation variant: {:?}", other),
    }
    assert!(report
        .guidance
        .contains("Set SUGGESTION_DELETE_THINKING_STORE = false"));
}

#[test]
fn test_architecture_refactor_without_prompt_is_flagged() {
    let config = ThinkingCacheConfig::routine("4.8.2");
    let report =
        evaluate_thinking_cache_configuration(&config, &ReleaseType::SchemaOrArchitectureRefactor);

    assert!(
        !report.is_compliant,
        "Architecture refactor without prompt must be flagged"
    );
    assert_eq!(report.violations.len(), 1);
    assert_eq!(
        report.violations[0],
        ThinkingCacheGuardViolation::ArchitectureRefactorMissingPrompt
    );
    assert!(report
        .guidance
        .contains("Set SUGGESTION_DELETE_THINKING_STORE = true"));
}

#[test]
fn test_compliant_architecture_refactor_with_matching_target_version_passes_clean() {
    let config = ThinkingCacheConfig::refactor("4.9.0", "4.9.0");
    let report =
        evaluate_thinking_cache_configuration(&config, &ReleaseType::SchemaOrArchitectureRefactor);

    assert!(
        report.is_compliant,
        "Architecture refactor with matching version must be compliant"
    );
    assert!(report.violations.is_empty());
    assert!(report.is_clean());
}

#[test]
fn test_target_version_mismatch_with_package_version_is_flagged() {
    let config = ThinkingCacheConfig::refactor("4.8.1", "4.8.2");
    let report =
        evaluate_thinking_cache_configuration(&config, &ReleaseType::SchemaOrArchitectureRefactor);

    assert!(
        !report.is_compliant,
        "Mismatched target version must be flagged"
    );
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ThinkingCacheGuardViolation::MismatchedTargetVersion {
            target_version,
            package_version,
        } => {
            assert_eq!(target_version, "4.8.1");
            assert_eq!(package_version, "4.8.2");
        }
        other => panic!("Unexpected violation variant: {:?}", other),
    }
}

#[test]
fn test_empty_target_version_on_enabled_prompt_is_flagged() {
    let config = ThinkingCacheConfig::new(true, None, "4.8.2");
    let report =
        evaluate_thinking_cache_configuration(&config, &ReleaseType::SchemaOrArchitectureRefactor);

    assert!(!report.is_compliant);
    assert_eq!(
        report.violations,
        vec![ThinkingCacheGuardViolation::EmptyTargetVersionOnEnabledPrompt]
    );

    let config_whitespace = ThinkingCacheConfig::new(true, Some("   ".to_string()), "4.8.2");
    let report_whitespace = evaluate_thinking_cache_configuration(
        &config_whitespace,
        &ReleaseType::SchemaOrArchitectureRefactor,
    );
    assert!(!report_whitespace.is_compliant);
    assert_eq!(
        report_whitespace.violations,
        vec![ThinkingCacheGuardViolation::EmptyTargetVersionOnEnabledPrompt]
    );
}

#[test]
fn test_version_prefix_normalization_passes() {
    let config = ThinkingCacheConfig::refactor("v4.8.2", "4.8.2");
    let report =
        evaluate_thinking_cache_configuration(&config, &ReleaseType::SchemaOrArchitectureRefactor);

    assert!(
        report.is_compliant,
        "Prefix 'v' in target_version should match package_version"
    );
    assert!(report.violations.is_empty());
}

#[test]
fn test_serde_roundtrip() {
    let config = ThinkingCacheConfig::refactor("4.8.2", "4.8.2");
    let serialized = serde_json::to_string(&config).expect("serialize config");
    let deserialized: ThinkingCacheConfig =
        serde_json::from_str(&serialized).expect("deserialize config");
    assert_eq!(config, deserialized);

    let release_type = ReleaseType::SchemaOrArchitectureRefactor;
    let serialized_rt = serde_json::to_string(&release_type).expect("serialize release_type");
    let deserialized_rt: ReleaseType =
        serde_json::from_str(&serialized_rt).expect("deserialize release_type");
    assert_eq!(release_type, deserialized_rt);

    let report = evaluate_thinking_cache_configuration(&config, &release_type);
    let serialized_report = serde_json::to_string(&report).expect("serialize report");
    let deserialized_report: ThinkingCacheReport =
        serde_json::from_str(&serialized_report).expect("deserialize report");
    assert_eq!(report, deserialized_report);
}

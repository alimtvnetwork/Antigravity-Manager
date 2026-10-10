// (split from variant_mapping.rs)
use super::resolve::{infer_tier, resolve, resolve_with_tier, tier_from_effort};
use super::test_helpers::check;
use super::types::VariantTier;

#[test]
fn claude_specs_preserve_the_client_budget() {
    assert!(SPEC_CLAUDE_SONNET_46.preserve_client_budget);
    assert!(SPEC_CLAUDE_OPUS_46.preserve_client_budget);
    assert_eq!(
        SPEC_CLAUDE_OPUS_46.effective_thinking_budget(Some(32_768)),
        32_768
    );
    assert_eq!(SPEC_CLAUDE_OPUS_46.effective_thinking_budget(None), 1_024);
}

#[test]
fn non_claude_specs_keep_the_server_budget() {
    assert!(!SPEC_35_FLASH_EXTRA_LOW.preserve_client_budget);
    assert!(!SPEC_35_FLASH_LOW.preserve_client_budget);
    assert!(!SPEC_3_FLASH_AGENT.preserve_client_budget);
    assert!(!SPEC_31_PRO_LOW.preserve_client_budget);
    assert!(!SPEC_PRO_AGENT.preserve_client_budget);
    assert!(!SPEC_31_FLASH_LITE.preserve_client_budget);
    assert!(!SPEC_GPT_OSS_120B.preserve_client_budget);
    assert_eq!(
        SPEC_3_FLASH_AGENT.effective_thinking_budget(Some(32_768)),
        10_000
    );
}

#[test]
fn test_infer_tier_ranges() {
    assert_eq!(infer_tier(None), VariantTier::High);
    assert_eq!(infer_tier(Some(0)), VariantTier::Low);
    assert_eq!(infer_tier(Some(1999)), VariantTier::Low);
    assert_eq!(infer_tier(Some(2000)), VariantTier::Medium);
    assert_eq!(infer_tier(Some(6999)), VariantTier::Medium);
    assert_eq!(infer_tier(Some(7000)), VariantTier::High);
    assert_eq!(infer_tier(Some(50000)), VariantTier::High);
}

#[test]
fn test_resolve_35_flash_variants() {
    // Medium (default)
    let s = resolve("gemini-3.5-flash", None).unwrap();
    assert_eq!(s.id, "gemini-3.5-flash-low");
    assert_eq!(s.thinking_budget, 4000);
    assert_eq!(s.max_output_tokens, 65536);

    // Medium
    let s = resolve("gemini-3.5-flash", Some(4000)).unwrap();
    assert_eq!(s.id, "gemini-3.5-flash-low");
    assert_eq!(s.thinking_budget, 4000);

    // [USER RULE] Bare model ignores budget (Some(1000)), returning Medium (4000)
    let s = resolve("gemini-3.5-flash", Some(1000)).unwrap();
    assert_eq!(s.id, "gemini-3.5-flash-low");
    assert_eq!(s.thinking_budget, 4000);

    // Low tier governed by explicit effort
    let s = resolve_with_tier("gemini-3.5-flash", Some(VariantTier::Low), None).unwrap();
    assert_eq!(s.id, "gemini-3.5-flash-extra-low");
    assert_eq!(s.thinking_budget, 1000);
}

#[test]
fn test_resolve_37_flash_variants() {
    // Suffix-less Flash models >= 3.6 route to tiered model ID
    let s = resolve("gemini-3.7-flash", None).unwrap();
    assert_eq!(s.id, "gemini-3.7-flash-tiered");
    assert_eq!(s.thinking_budget, 4000);
    assert_eq!(s.max_output_tokens, 65536);

    // High
    let s_high = resolve("gemini-3.7-flash-high", None).unwrap();
    assert_eq!(s_high.id, "gemini-3.7-flash-high");
    assert_eq!(s_high.thinking_budget, 10000);

    // Medium
    let s_med = resolve("gemini-3.7-flash-medium", None).unwrap();
    assert_eq!(s_med.id, "gemini-3.7-flash-medium");
    assert_eq!(s_med.thinking_budget, 4000);

    // Low
    let s_low = resolve("gemini-3.7-flash-low", None).unwrap();
    assert_eq!(s_low.id, "gemini-3.7-flash-low");
    assert_eq!(s_low.thinking_budget, 1000);
}

#[test]
fn test_resolve_37_flash_high_never_downgraded_by_budget() {
    for budget in [None, Some(0), Some(1000), Some(1024), Some(4000)] {
        let s = resolve("gemini-3.7-flash-high", budget).unwrap();
        assert_eq!(s.id, "gemini-3.7-flash-high");
        assert_eq!(s.thinking_budget, 10000);
    }
}

#[test]
fn test_resolve_38_flash_variants() {
    // Suffix-less Flash models >= 3.6 route to tiered model ID
    let s = resolve("gemini-3.8-flash", None).unwrap();
    assert_eq!(s.id, "gemini-3.8-flash-tiered");
    assert_eq!(s.thinking_budget, 4000);
    assert_eq!(s.max_output_tokens, 65536);

    // High
    let s_high = resolve("gemini-3.8-flash-high", None).unwrap();
    assert_eq!(s_high.id, "gemini-3.8-flash-high");
    assert_eq!(s_high.thinking_budget, 10000);

    // Medium
    let s_med = resolve("gemini-3.8-flash-medium", None).unwrap();
    assert_eq!(s_med.id, "gemini-3.8-flash-medium");
    assert_eq!(s_med.thinking_budget, 4000);

    // Low
    let s_low = resolve("gemini-3.8-flash-low", None).unwrap();
    assert_eq!(s_low.id, "gemini-3.8-flash-low");
    assert_eq!(s_low.thinking_budget, 1000);

    // High never downgraded
    for budget in [None, Some(0), Some(1000), Some(1024)] {
        let s = resolve("gemini-3.8-flash-high", budget).unwrap();
        assert_eq!(s.id, "gemini-3.8-flash-high");
        assert_eq!(s.thinking_budget, 10000);
    }
}

#[test]
fn test_dynamic_unregistered_gemini_3_family() {
    // Any unregistered Gemini >= 3 model resolves dynamically without hardcoded registry
    // Suffix-less Flash models route to tiered model ID
    let s = resolve("gemini-3.9-flash", None).unwrap();
    assert_eq!(s.id, "gemini-3.9-flash-tiered");
    assert_eq!(s.thinking_budget, 4000);
    assert_eq!(s.max_output_tokens, 65536);

    // Explicit tier in name
    let s_med = resolve("gemini-3.9-flash-medium", None).unwrap();
    assert_eq!(s_med.id, "gemini-3.9-flash-medium");
    assert_eq!(s_med.thinking_budget, 4000);

    let s_low = resolve("gemini-3.9-flash-low", None).unwrap();
    assert_eq!(s_low.id, "gemini-3.9-flash-low");
    assert_eq!(s_low.thinking_budget, 1000);

    // Budget never downgrades unregistered -high model
    for budget in [None, Some(0), Some(1000), Some(1024)] {
        let s = resolve("gemini-3.9-flash-high", budget).unwrap();
        assert_eq!(s.id, "gemini-3.9-flash-high");
        assert_eq!(s.thinking_budget, 10000);
    }

    // Pro model defaults to 10001
    let s = resolve("gemini-3.9-pro", None).unwrap();
    assert_eq!(s.id, "gemini-3.9-pro");
    assert_eq!(s.thinking_budget, 10001);
}

#[test]
fn tier_from_effort_accepts_only_anthropic_sdk_values() {
    assert_eq!(tier_from_effort(Some("low")), Some(VariantTier::Low));
    assert_eq!(tier_from_effort(Some("extra-low")), Some(VariantTier::Low));
    assert_eq!(tier_from_effort(Some("medium")), Some(VariantTier::Medium));
    assert_eq!(tier_from_effort(Some("default")), Some(VariantTier::Medium));
    assert_eq!(tier_from_effort(Some("high")), Some(VariantTier::High));
    assert_eq!(tier_from_effort(Some("max")), Some(VariantTier::High));
    assert_eq!(tier_from_effort(Some("xhigh")), Some(VariantTier::High));
    assert_eq!(tier_from_effort(Some("unknown")), None);
    assert_eq!(tier_from_effort(None), None);
}

#[test]
fn resolve_with_tier_prefers_explicit_effort_over_budget() {
    for (canonical, tier, budget, expected_id) in [
        (
            "gemini-3.5-flash",
            VariantTier::Low,
            10_000,
            "gemini-3.5-flash-extra-low",
        ),
        (
            "gemini-3.5-flash",
            VariantTier::Medium,
            1_000,
            "gemini-3.5-flash-low",
        ),
        (
            "gemini-3.5-flash",
            VariantTier::High,
            1_000,
            "gemini-3-flash-agent",
        ),
        (
            "gemini-3.1-pro",
            VariantTier::Low,
            10_000,
            "gemini-3.1-pro-low",
        ),
        (
            "gemini-3.1-pro",
            VariantTier::Medium,
            1_000,
            "gemini-pro-agent",
        ),
        (
            "gemini-3.1-pro",
            VariantTier::High,
            1_000,
            "gemini-pro-agent",
        ),
    ] {
        let spec = resolve_with_tier(canonical, Some(tier), Some(budget))
            .expect("canonical Gemini tier must resolve");
        assert_eq!(spec.id, expected_id);
    }
}

#[test]
fn gemini_3_flash_alias_follows_tier() {
    // Bare model budget is ignored, governed by reasoning effort; defaults to Medium
    check(
        "gemini-3-flash",
        Some(0),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3-flash",
        Some(4000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check("gemini-3-flash", None, "gemini-3.5-flash-low", 4000, 65536);

    // Explicit effort tier retains strict precedence
    let low_spec = resolve_with_tier("gemini-3-flash", Some(VariantTier::Low), None).unwrap();
    assert_eq!(low_spec.id, "gemini-3.5-flash-extra-low");
    let high_spec = resolve_with_tier("gemini-3-flash", Some(VariantTier::High), None).unwrap();
    assert_eq!(high_spec.id, "gemini-3-flash-agent");
}

#[test]
fn test_resolve_31_pro_variants() {
    let s = resolve("gemini-3.1-pro", None).unwrap();
    assert_eq!(s.id, "gemini-pro-agent");
    assert_eq!(s.thinking_budget, 10001);
    assert_eq!(s.max_output_tokens, 65535); // Pro uses 65535, not 65536

    // Bare model budget is ignored, defaults to Medium (10001)
    let s = resolve("gemini-3.1-pro", Some(1001)).unwrap();
    assert_eq!(s.id, "gemini-pro-agent");
    assert_eq!(s.thinking_budget, 10001);

    // Explicit effort Low takes precedence
    let low = resolve_with_tier("gemini-3.1-pro", Some(VariantTier::Low), None).unwrap();
    assert_eq!(low.id, "gemini-3.1-pro-low");
    assert_eq!(low.thinking_budget, 1001);
}

#[test]
fn test_resolve_non_variant_models() {
    let s = resolve("gemini-3.1-flash-lite", None).unwrap();
    assert_eq!(s.id, "gemini-3.1-flash-lite");
    assert_eq!(s.thinking_budget, 0);
    assert!(!s.include_thoughts);

    let s = resolve("claude-sonnet-4-6", None).unwrap();
    assert_eq!(s.id, "claude-sonnet-4-6");
    assert_eq!(s.thinking_budget, 1024);
    assert_eq!(s.max_output_tokens, 64000);
}

#[test]
fn test_unknown_model_returns_none() {
    assert!(resolve("some-unknown-model", None).is_none());
}

#[test]
fn test_case_insensitive() {
    let s = resolve("GEMINI-3.5-FLASH", None).unwrap();
    assert_eq!(s.id, "gemini-3.5-flash-low");
}

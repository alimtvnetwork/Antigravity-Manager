// (split from variant_mapping.rs)
use super::resolve::{infer_tier, resolve, resolve_with_tier, tier_from_effort};
use super::test_helpers::check;
use super::types::VariantTier;

#[test]
fn baseline_resolve_35_flash_variants() {
    // ── gemini-3.5-flash (canonical) ───────────────────────────────
    // None → Medium → gemini-3.5-flash-low
    check(
        "gemini-3.5-flash",
        None,
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    // [USER RULE] Bare model ignores client budget, defaults to Medium (4000)
    check(
        "gemini-3.5-flash",
        Some(0),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3.5-flash",
        Some(4000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3.5-flash",
        Some(7000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );

    // ── gemini-3.5-flash-high ──────────────────────────────────────
    // Explicit -high locks highest tier, never downgrades
    check(
        "gemini-3.5-flash-high",
        None,
        "gemini-3-flash-agent",
        10000,
        65536,
    );
    check(
        "gemini-3.5-flash-high",
        Some(0),
        "gemini-3-flash-agent",
        10000,
        65536,
    );
    check(
        "gemini-3.5-flash-high",
        Some(4000),
        "gemini-3-flash-agent",
        10000,
        65536,
    );
    check(
        "gemini-3.5-flash-high",
        Some(7000),
        "gemini-3-flash-agent",
        10000,
        65536,
    );

    // ── gemini-3.5-flash-medium (fixed → SPEC_35_FLASH_LOW) ────────
    check(
        "gemini-3.5-flash-medium",
        None,
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3.5-flash-medium",
        Some(0),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3.5-flash-medium",
        Some(4000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "gemini-3.5-flash-medium",
        Some(7000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );

    // ── gemini-3.5-flash-low (fixed → SPEC_35_FLASH_EXTRA_LOW) ─────
    check(
        "gemini-3.5-flash-low",
        None,
        "gemini-3.5-flash-extra-low",
        1000,
        65536,
    );
    check(
        "gemini-3.5-flash-low",
        Some(0),
        "gemini-3.5-flash-extra-low",
        1000,
        65536,
    );
    check(
        "gemini-3.5-flash-low",
        Some(4000),
        "gemini-3.5-flash-extra-low",
        1000,
        65536,
    );
    check(
        "gemini-3.5-flash-low",
        Some(7000),
        "gemini-3.5-flash-extra-low",
        1000,
        65536,
    );
}

#[test]
fn baseline_resolve_31_pro_variants() {
    // ── gemini-3.1-pro (canonical) ─────────────────────────────────
    // None → High → gemini-pro-agent
    check("gemini-3.1-pro", None, "gemini-pro-agent", 10001, 65535);
    // Some(0) → budget ignored → Medium/High → gemini-pro-agent
    check("gemini-3.1-pro", Some(0), "gemini-pro-agent", 10001, 65535);
    // Some(4000) → Medium → High fallback → gemini-pro-agent
    check(
        "gemini-3.1-pro",
        Some(4000),
        "gemini-pro-agent",
        10001,
        65535,
    );
    // Some(7000) → High → gemini-pro-agent
    check(
        "gemini-3.1-pro",
        Some(7000),
        "gemini-pro-agent",
        10001,
        65535,
    );

    // ── gemini-3.1-pro-high (same canonical arm) ───────────────────
    check(
        "gemini-3.1-pro-high",
        None,
        "gemini-pro-agent",
        10001,
        65535,
    );
    check(
        "gemini-3.1-pro-high",
        Some(0),
        "gemini-pro-agent",
        10001,
        65535,
    );
    check(
        "gemini-3.1-pro-high",
        Some(4000),
        "gemini-pro-agent",
        10001,
        65535,
    );
    check(
        "gemini-3.1-pro-high",
        Some(7000),
        "gemini-pro-agent",
        10001,
        65535,
    );

    // ── gemini-pro (same canonical arm) ────────────────────────────
    check("gemini-pro", None, "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(0), "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(4000), "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(7000), "gemini-pro-agent", 10001, 65535);

    // ── gemini-3.1-pro-low (fixed → SPEC_31_PRO_LOW) ──────────────
    check(
        "gemini-3.1-pro-low",
        None,
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(0),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(4000),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(7000),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
}

#[test]
fn baseline_resolve_non_variant_models() {
    // Gemini flash-lite family – all map to SPEC_31_FLASH_LITE
    let check_lite = |c: &str| {
        check(c, None, "gemini-3.1-flash-lite", 0, 16384);
        // budget should not affect non-variant resolution
        check(c, Some(0), "gemini-3.1-flash-lite", 0, 16384);
        check(c, Some(7000), "gemini-3.1-flash-lite", 0, 16384);
    };
    check_lite("gemini-3.1-flash-lite");
    check_lite("gemini-2.5-flash-lite");
    check_lite("gemini-2.5-flash");
    check_lite("gemini-2.5-flash-thinking");

    // ── Claude family ──────────────────────────────────────────────
    check("claude-sonnet-4-6", None, "claude-sonnet-4-6", 1024, 64000);
    // claude-opus-4-6-thinking and claude-opus-4-6 → same spec
    check(
        "claude-opus-4-6-thinking",
        None,
        "claude-opus-4-6-thinking",
        1024,
        64000,
    );
    check(
        "claude-opus-4-6",
        None,
        "claude-opus-4-6-thinking",
        1024,
        64000,
    );

    // ── GPT OSS ────────────────────────────────────────────────────
    check(
        "gpt-oss-120b-medium",
        None,
        "gpt-oss-120b-medium",
        8192,
        32768,
    );
}

#[test]
fn baseline_unknown_model() {
    assert!(resolve("some-unknown-model", None).is_none());
    assert!(resolve("gemini-4.0-super", Some(4000)).is_none());
    assert!(resolve("", None).is_none());
}

#[test]
fn baseline_case_insensitive() {
    // Canonical variant split model
    check(
        "GEMINI-3.5-FLASH",
        None,
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check(
        "Gemini-3.5-Flash",
        None,
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
    check("GEMINI-3.1-PRO", None, "gemini-pro-agent", 10001, 65535);
    // Non-variant model
    check("CLAUDE-SONNET-4-6", None, "claude-sonnet-4-6", 1024, 64000);
    check(
        "GPT-OSS-120B-MEDIUM",
        None,
        "gpt-oss-120b-medium",
        8192,
        32768,
    );
}

// ═════════════════════════════════════════════════════════════════════
// old catalog id fallback — these legacy top-level ids must keep
// resolving through their family aliases.  See task-5-brief.md.
// ═════════════════════════════════════════════════════════════════════

#[test]
fn old_catalog_alias_fallback() {
    // ── gemini-3.1-pro-high (Fixed(High) → always gemini-pro-agent, never downgraded) ────
    // None → High → gemini-pro-agent
    check(
        "gemini-3.1-pro-high",
        None,
        "gemini-pro-agent",
        10001,
        65535,
    );
    // Some(0) → High (Fixed, no downgrade) → gemini-pro-agent
    check(
        "gemini-3.1-pro-high",
        Some(0),
        "gemini-pro-agent",
        10001,
        65535,
    );
    // Some(4000) → High (Fixed, no downgrade) → gemini-pro-agent
    check(
        "gemini-3.1-pro-high",
        Some(4000),
        "gemini-pro-agent",
        10001,
        65535,
    );
    // Some(7000) → High → gemini-pro-agent
    check(
        "gemini-3.1-pro-high",
        Some(7000),
        "gemini-pro-agent",
        10001,
        65535,
    );

    // ── gemini-3.1-pro-low (Fixed(Low) → always gemini-3.1-pro-low)
    check(
        "gemini-3.1-pro-low",
        None,
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(0),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(4000),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );
    check(
        "gemini-3.1-pro-low",
        Some(7000),
        "gemini-3.1-pro-low",
        1001,
        65535,
    );

    // ── gemini-pro (HonorTier → same as gemini-3.1-pro) ────────────
    check("gemini-pro", None, "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(0), "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(4000), "gemini-pro-agent", 10001, 65535);
    check("gemini-pro", Some(7000), "gemini-pro-agent", 10001, 65535);

    // ── gemini-3-flash (HonorTier → same as gemini-3.5-flash) ──────
    check("gemini-3-flash", None, "gemini-3.5-flash-low", 4000, 65536);
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
    check(
        "gemini-3-flash",
        Some(7000),
        "gemini-3.5-flash-low",
        4000,
        65536,
    );
}

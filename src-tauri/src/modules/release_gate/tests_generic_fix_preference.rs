//! Unit tests for Generic-Fix Preference & Future-Proofing Guard.

use crate::modules::release_gate::generic_fix_preference_guard::{
    derive_recommended_regex, derive_recommended_wildcard, evaluate_match_sites,
    evaluate_single_site, GeneralizationFinding, GeneralizationReport, GeneralizationSuggestion,
    MatchSite, MatchSiteKind, EXACT_CHAIN_THRESHOLD, LITERAL_TABLE_THRESHOLD,
};

#[test]
fn test_five_branch_exact_chain_without_justification_is_flagged() {
    let site = MatchSite::exact_chain("model_dispatch_ladder", 5)
        .with_samples(vec!["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash"]);

    let report = evaluate_match_sites(&[site]);
    assert!(!report.is_clean, "Five-branch exact chain must be flagged");
    assert_eq!(report.total_sites_evaluated, 1);
    assert_eq!(report.generalization_required_count, 1);

    let finding = &report.findings[0];
    assert!(finding.is_generalization_required);
    assert_eq!(finding.site_id, "model_dispatch_ladder");
    assert!(finding.explanation.contains("exceeds threshold"));
    assert!(matches!(
        finding.suggested_generalization,
        GeneralizationSuggestion::UseWildcardGlob { .. }
    ));
}

#[test]
fn test_exact_chain_with_justification_passes_clean() {
    let site = MatchSite::exact_chain("core_protocol_adapter_switch", 5)
        .with_justification("Protocol adapters are closed set normalized to 4 protocols per AGENTS.md");

    let report = evaluate_match_sites(&[site]);
    assert!(report.is_clean, "Justified exact chain must pass clean");
    assert_eq!(report.generalization_required_count, 0);

    let finding = &report.findings[0];
    assert!(!finding.is_generalization_required);
    assert_eq!(
        finding.suggested_generalization,
        GeneralizationSuggestion::JustifiedExemptionAcknowledged
    );
    assert!(finding.suggested_generalization.is_acknowledged());
}

#[test]
fn test_small_exact_chain_below_threshold_passes() {
    let site = MatchSite::exact_chain("two_way_branch", EXACT_CHAIN_THRESHOLD - 1);
    let report = evaluate_match_sites(&[site]);
    assert!(
        report.is_clean,
        "Exact chain below threshold must pass without justification"
    );
    assert_eq!(report.generalization_required_count, 0);
    assert!(!report.findings[0].is_generalization_required);
}

#[test]
fn test_wildcard_pattern_matcher_passes_clean() {
    let site = MatchSite::wildcard("gemini_wildcard_matcher")
        .with_sample("gemini-*-flash-*");

    let report = evaluate_match_sites(&[site]);
    assert!(report.is_clean, "Wildcard pattern must pass clean");
    assert_eq!(report.generalization_required_count, 0);

    let finding = &report.findings[0];
    assert!(!finding.is_generalization_required);
    assert!(finding.is_compliant());
    assert!(matches!(
        finding.suggested_generalization,
        GeneralizationSuggestion::UseWildcardGlob { .. }
    ));
}

#[test]
fn test_regex_pattern_matcher_passes_clean() {
    let site = MatchSite::regex("prompt_sanitizer_regex")
        .with_sample(r"^sk-[a-zA-Z0-9]{20,}$");

    let report = evaluate_match_sites(&[site]);
    assert!(report.is_clean, "Regex pattern must pass clean");
    assert_eq!(report.generalization_required_count, 0);

    let finding = &report.findings[0];
    assert!(!finding.is_generalization_required);
    assert!(matches!(
        finding.suggested_generalization,
        GeneralizationSuggestion::UseRegexEngine { .. }
    ));
}

#[test]
fn test_registry_lookup_passes_clean() {
    let site = MatchSite::registry("dynamic_adapter_registry");

    let report = evaluate_match_sites(&[site]);
    assert!(report.is_clean, "Registry lookup must pass clean");
    assert_eq!(report.generalization_required_count, 0);

    let finding = &report.findings[0];
    assert!(!finding.is_generalization_required);
    assert_eq!(
        finding.suggested_generalization,
        GeneralizationSuggestion::UseDynamicRegistry
    );
}

#[test]
fn test_single_exact_match_with_justification_passes_clean() {
    let site = MatchSite::single_exact("root_endpoint_probe")
        .with_justification("Root probe is uniquely defined by HTTP RFC 9110 specification");

    let report = evaluate_match_sites(&[site]);
    assert!(
        report.is_clean,
        "Single exact match with justification must pass clean"
    );
    assert_eq!(report.generalization_required_count, 0);

    let finding = &report.findings[0];
    assert!(!finding.is_generalization_required);
    assert_eq!(
        finding.suggested_generalization,
        GeneralizationSuggestion::JustifiedExemptionAcknowledged
    );
}

#[test]
fn test_single_exact_match_without_justification_is_flagged() {
    let site = MatchSite::single_exact("hardcoded_model_check")
        .with_sample("gemini-1.5-flash");

    let report = evaluate_match_sites(&[site]);
    assert!(
        !report.is_clean,
        "Single exact match without justification must be flagged"
    );
    assert_eq!(report.generalization_required_count, 1);

    let finding = &report.findings[0];
    assert!(finding.is_generalization_required);
    assert!(matches!(
        finding.suggested_generalization,
        GeneralizationSuggestion::UseWildcardGlob { .. }
    ));
}

#[test]
fn test_literal_table_flagged_without_justification_and_acknowledged_with() {
    // Unjustified literal table with 10 entries (>= 4 threshold)
    let unjustified = MatchSite::literal_table("model_adapter_dictionary", 10);
    let report1 = evaluate_match_sites(&[unjustified]);
    assert!(
        !report1.is_clean,
        "Literal table with many entries must be flagged without justification"
    );
    assert_eq!(report1.generalization_required_count, 1);
    assert_eq!(
        report1.findings[0].suggested_generalization,
        GeneralizationSuggestion::UseDynamicRegistry
    );

    // Justified literal table with 10 entries
    let justified = MatchSite::literal_table("mime_types_map", 10)
        .with_justification("Standard HTTP Content-Type MIME lookup per RFC 2046");
    let report2 = evaluate_match_sites(&[justified]);
    assert!(
        report2.is_clean,
        "Literal table must pass clean when justification is provided"
    );
    assert_eq!(report2.generalization_required_count, 0);
    assert_eq!(
        report2.findings[0].suggested_generalization,
        GeneralizationSuggestion::JustifiedExemptionAcknowledged
    );

    // Literal table below threshold (< 4 entries)
    let small_table = MatchSite::literal_table("small_protocol_set", LITERAL_TABLE_THRESHOLD - 1);
    let report3 = evaluate_match_sites(&[small_table]);
    assert!(
        report3.is_clean,
        "Literal table below threshold must pass without justification"
    );
    assert_eq!(report3.generalization_required_count, 0);
}

#[test]
fn test_serde_roundtrip() {
    // 1. MatchSiteKind roundtrip
    let kinds = vec![
        MatchSiteKind::ExactEqualityChain { branch_count: 5 },
        MatchSiteKind::LiteralLookupTable { entry_count: 8 },
        MatchSiteKind::WildcardPattern,
        MatchSiteKind::RegexPattern,
        MatchSiteKind::RegistryLookup,
        MatchSiteKind::SingleExactMatch,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).expect("serialize kind");
        let decoded: MatchSiteKind = serde_json::from_str(&json).expect("deserialize kind");
        assert_eq!(kind, &decoded);
    }

    // 2. MatchSite roundtrip
    let site = MatchSite::exact_chain("site_alpha", 4)
        .with_samples(vec!["sample_1", "sample_2"])
        .with_justification("Test exemption");
    let site_json = serde_json::to_string(&site).expect("serialize site");
    let decoded_site: MatchSite = serde_json::from_str(&site_json).expect("deserialize site");
    assert_eq!(site, decoded_site);

    // 3. GeneralizationSuggestion roundtrip
    let suggestions = vec![
        GeneralizationSuggestion::UseWildcardGlob {
            recommended_pattern: "gemini-*-flash-*".to_string(),
        },
        GeneralizationSuggestion::UseRegexEngine {
            recommended_pattern: r"^gpt-4o-.*$".to_string(),
        },
        GeneralizationSuggestion::UseDynamicRegistry,
        GeneralizationSuggestion::JustifiedExemptionAcknowledged,
    ];
    for suggestion in &suggestions {
        let json = serde_json::to_string(suggestion).expect("serialize suggestion");
        let decoded: GeneralizationSuggestion =
            serde_json::from_str(&json).expect("deserialize suggestion");
        assert_eq!(suggestion, &decoded);
    }

    // 4. GeneralizationFinding roundtrip
    let finding = evaluate_single_site(&site);
    let finding_json = serde_json::to_string(&finding).expect("serialize finding");
    let decoded_finding: GeneralizationFinding =
        serde_json::from_str(&finding_json).expect("deserialize finding");
    assert_eq!(finding, decoded_finding);

    // 5. GeneralizationReport roundtrip
    let report = evaluate_match_sites(&[site]);
    let report_json = serde_json::to_string(&report).expect("serialize report");
    let decoded_report: GeneralizationReport =
        serde_json::from_str(&report_json).expect("deserialize report");
    assert_eq!(report, decoded_report);
}

#[test]
fn test_multi_site_batch_evaluation() {
    let sites = vec![
        // 1. Exact chain with 5 branches (unjustified -> flagged)
        MatchSite::exact_chain("chain_large", 5),
        // 2. Wildcard pattern (compliant -> pass)
        MatchSite::wildcard("wildcard_gemini"),
        // 3. Literal table (10 entries, justified -> pass)
        MatchSite::literal_table("table_justified", 10).with_justification("Static RFC mapping"),
        // 4. Literal table (6 entries, unjustified -> flagged)
        MatchSite::literal_table("table_unjustified", 6),
        // 5. Single exact match (justified -> pass)
        MatchSite::single_exact("single_justified").with_justification("Singleton endpoint"),
    ];

    let report = evaluate_match_sites(&sites);
    assert!(!report.is_clean);
    assert_eq!(report.total_sites_evaluated, 5);
    assert_eq!(report.generalization_required_count, 2);
    assert_eq!(report.required_findings().len(), 2);
    assert_eq!(report.compliant_findings().len(), 3);
}

#[test]
fn test_derive_recommended_patterns() {
    // Single sample with multiple hyphens
    let pat1 = derive_recommended_wildcard(&["gemini-1.5-flash".to_string()]);
    assert_eq!(pat1, "gemini-*-flash*");

    // Single sample with two parts
    let pat2 = derive_recommended_wildcard(&["claude-3".to_string()]);
    assert_eq!(pat2, "claude-*");

    // Multiple samples sharing prefix and suffix
    let pat3 = derive_recommended_wildcard(&[
        "gemini-1.5-flash".to_string(),
        "gemini-2.0-flash".to_string(),
    ]);
    assert_eq!(pat3, "gemini-*-flash");

    // Empty samples fallback
    assert_eq!(derive_recommended_wildcard(&[]), "*");

    // Regex derivation
    let regex_pat = derive_recommended_regex(&["gemini-1.5-pro".to_string()]);
    assert!(regex_pat.starts_with('^'));
    assert!(regex_pat.ends_with('$'));
}

#[test]
fn test_format_markdown_summary() {
    let clean_report = GeneralizationReport::empty();
    let clean_summary = clean_report.format_markdown_summary();
    assert!(clean_summary.contains("🟢 **Status**: PASS"));
    assert!(clean_summary.contains("| Site ID | Required |"));

    let flagged_site = MatchSite::exact_chain("hardcoded_ladder", 5);
    let flagged_report = evaluate_match_sites(&[flagged_site]);
    let flagged_summary = flagged_report.format_markdown_summary();
    assert!(flagged_summary.contains("🔴 **Status**: ACTION REQUIRED"));
    assert!(flagged_summary.contains("⚠️ Yes"));
}

#[test]
fn test_match_site_builder_methods() {
    let site = MatchSite::new("test_site", MatchSiteKind::SingleExactMatch)
        .with_sample("sample_a")
        .with_samples(vec!["sample_b", "sample_c"])
        .with_justification("Testing justification");

    assert_eq!(site.site_id, "test_site");
    assert_eq!(site.literal_samples, vec!["sample_b", "sample_c"]);
    assert!(site.has_justification);
    assert_eq!(site.justification.as_deref(), Some("Testing justification"));

    let reset_site = site.without_justification();
    assert!(!reset_site.has_justification);
    assert_eq!(reset_site.justification, None);
}

#[test]
fn test_match_site_kind_predicates_and_display() {
    let chain = MatchSiteKind::ExactEqualityChain { branch_count: 4 };
    assert!(!chain.is_future_proof());
    assert!(chain.is_exact_match());
    assert!(!chain.is_literal_table());
    assert_eq!(chain.to_string(), "Exact equality chain with 4 branches");

    let table = MatchSiteKind::LiteralLookupTable { entry_count: 5 };
    assert!(!table.is_future_proof());
    assert!(!table.is_exact_match());
    assert!(table.is_literal_table());
    assert_eq!(table.to_string(), "Literal lookup table with 5 entries");

    let wildcard = MatchSiteKind::WildcardPattern;
    assert!(wildcard.is_future_proof());
    assert!(!wildcard.is_exact_match());
    assert_eq!(wildcard.to_string(), "Wildcard pattern glob");

    let regex = MatchSiteKind::RegexPattern;
    assert!(regex.is_future_proof());
    assert_eq!(regex.to_string(), "Regular expression engine");

    let registry = MatchSiteKind::RegistryLookup;
    assert!(registry.is_future_proof());
    assert_eq!(registry.to_string(), "Dynamic registry lookup");

    let single = MatchSiteKind::SingleExactMatch;
    assert!(!single.is_future_proof());
    assert!(single.is_exact_match());
    assert_eq!(single.to_string(), "Single exact match comparison");
}

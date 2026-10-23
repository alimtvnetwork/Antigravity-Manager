//! Unit tests for Bounded Matcher & Timeout Verifier (Imp-664).
//!
//! File: src-tauri/src/modules/release_gate/tests_bounded_matcher_timeout_verifier.rs

#[cfg(test)]
mod tests {
    use super::super::bounded_matcher_timeout_verifier::*;
    use std::time::Duration;

    #[test]
    fn test_single_line_bounded_patterns_compliant() {
        let patterns = [
            r"^[a-zA-Z0-9_-]+$",
            r"^v\d+\.\d+\.\d+$",
            r"\[v[0-9.]+\]",
            r"fn\s+[a-z_][a-z0-9_]*",
            r"class\s+([A-Za-z0-9_]+)",
            r"#([0-9a-fA-F]{6})",
            r"<[^>]*>",
        ];
        let verifier = BoundedMatcherTimeoutVerifier::new();
        for pattern in patterns {
            let res = verifier.audit_pattern(pattern);
            assert!(res.is_ok(), "Pattern should be compliant: {pattern}");
            assert!(verify_matcher_pattern_bounded(pattern).is_ok());
        }
    }

    #[test]
    fn test_multiline_dot_all_unbounded_patterns_flagged() {
        let patterns = [
            r"(?s).*",
            r"(?s).+",
            r"(?s:.*)",
            r"[\s\S]*",
            r"[\s\S]+",
            r"[\w\W]*",
            r"[\d\D]*",
        ];
        let verifier = BoundedMatcherTimeoutVerifier::new();
        for pattern in patterns {
            let res = verifier.audit_pattern(pattern);
            assert!(res.is_err(), "Multiline dot-all must be flagged: {pattern}");
        }
    }

    #[test]
    fn test_bare_unanchored_wildcards_flagged() {
        let patterns = [r".*", r".+", r"^.*$", r"^.+$", r"(.*)", r"(.+)"];
        let verifier = BoundedMatcherTimeoutVerifier::new();
        for pattern in patterns {
            let res = verifier.audit_pattern(pattern);
            assert!(res.is_err(), "Bare wildcard must be flagged: {pattern}");
        }
    }

    #[test]
    fn test_tag_boundary_crossing_patterns_flagged() {
        let patterns = [
            r"<.*>",
            r"<.+>",
            r"<.*?>",
            r"<.+?>",
            r"<[^>]*.*>",
            r"<.*[^>]*>",
        ];
        let verifier = BoundedMatcherTimeoutVerifier::new();
        for pattern in patterns {
            let res = verifier.audit_pattern(pattern);
            assert!(res.is_err(), "Tag boundary crossing must be flagged: {pattern}");
        }
    }

    #[test]
    fn test_zero_timeout_rejection() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let zero = Duration::ZERO;
        let res_polling = verifier.validate_timeout(zero, TimeoutValidationType::PollingLoop);
        assert!(res_polling.is_err(), "Zero timeout in polling loop must fail");
        let res_retry = verifier.validate_timeout(zero, TimeoutValidationType::RetryLadder);
        assert!(res_retry.is_err(), "Zero timeout in retry ladder must fail");
        let res_net = verifier.validate_timeout(zero, TimeoutValidationType::NetworkWait);
        assert!(res_net.is_err(), "Zero timeout in network wait must fail");
        let res_fn = verify_wait_timeout(0, 30_000);
        assert!(res_fn.is_err(), "Zero timeout in verify_wait_timeout must fail");
    }

    #[test]
    fn test_safe_bounded_timeout_validation() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let d50 = Duration::from_millis(50);
        let d5000 = Duration::from_millis(5_000);
        let d60k = Duration::from_millis(60_000);
        assert!(verifier.validate_timeout(d50, TimeoutValidationType::PollingLoop).is_ok());
        assert!(verifier.validate_timeout(d5000, TimeoutValidationType::RetryLadder).is_ok());
        assert!(verifier.validate_timeout(d60k, TimeoutValidationType::NetworkWait).is_ok());
        assert!(verify_wait_timeout(50, 1000).is_ok());
        assert!(verify_wait_timeout(5000, 30000).is_ok());
    }

    #[test]
    fn test_timeout_exceeding_ceiling_rejection() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let excessive = Duration::from_millis(60_001);
        let res = verifier.validate_timeout(excessive, TimeoutValidationType::NetworkWait);
        assert!(res.is_err(), "Timeout above ceiling must fail");
        let res_cap = verify_wait_timeout(30_001, 30_000);
        assert!(res_cap.is_err(), "Timeout exceeding cap must fail");
        let res_huge = verify_wait_timeout(100_000, 5000);
        assert!(res_huge.is_err(), "Huge timeout exceeding cap must fail");
    }

    #[test]
    fn test_timeout_below_floor_rejection() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let low = Duration::from_millis(49);
        let res = verifier.validate_timeout(low, TimeoutValidationType::PollingLoop);
        assert!(res.is_err(), "Timeout below floor must fail");
        let res_fn = verify_wait_timeout(10, 1000);
        assert!(res_fn.is_err(), "Timeout below floor in helper must fail");
    }

    #[test]
    fn test_report_generation_and_serializability() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let patterns = [r"^[a-z]+$", r"(?s).*"];
        let timeouts = [
            (Duration::from_millis(1000), TimeoutValidationType::PollingLoop),
            (Duration::ZERO, TimeoutValidationType::NetworkWait),
        ];
        let report = verifier.audit_all(&patterns, &timeouts);
        assert_eq!(report.violation_count(), 2);
        let serialized = serde_json::to_string(&report).expect("must serialize");
        let deserialized: BoundedMatcherAuditReport =
            serde_json::from_str(&serialized).expect("must deserialize");
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_report_positive_booleans_state() {
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let clean_patterns = [r"^[a-z0-9_]+$"];
        let clean_timeouts = [(Duration::from_millis(500), TimeoutValidationType::PollingLoop)];
        let report = verifier.audit_all(&clean_patterns, &clean_timeouts);
        assert!(report.is_compliant());
        assert!(report.has_valid_boundaries());
        assert!(report.has_valid_timeouts());
        assert_eq!(report.violation_count(), 0);
        let def = BoundedMatcherAuditReport::default();
        assert!(def.is_compliant());
    }

    #[test]
    fn test_verifier_with_bounds_and_app_error() {
        let zero_res = BoundedMatcherTimeoutVerifier::with_bounds(
            Duration::ZERO,
            Duration::from_millis(1000),
        );
        assert!(zero_res.is_err());
        let inv_res = BoundedMatcherTimeoutVerifier::with_bounds(
            Duration::from_millis(500),
            Duration::from_millis(100),
        );
        assert!(inv_res.is_err());
        let verifier = BoundedMatcherTimeoutVerifier::new();
        let err_res = verifier.verify_and_enforce(&[r"(?s).*"], &[]);
        assert!(err_res.is_err());
    }

    #[test]
    fn test_snippet_boundaries_validation() {
        let ok_line = verify_matched_snippet("hello world", MatcherBoundaryType::SingleLineOnly);
        assert!(ok_line.is_ok());
        let err_nl = verify_matched_snippet("hello\nworld", MatcherBoundaryType::SingleLineOnly);
        assert!(err_nl.is_err());
        let ok_tag = verify_matched_snippet("inside tag", MatcherBoundaryType::WithinTagBound);
        assert!(ok_tag.is_ok());
        let err_tag = verify_matched_snippet("tag<div>bleed", MatcherBoundaryType::WithinTagBound);
        assert!(err_tag.is_err());
        let err_clause = verify_matched_snippet("a = 1; b = 2", MatcherBoundaryType::ClauseBounded);
        assert!(err_clause.is_err());
    }

    #[test]
    fn test_snippet_span_limit() {
        let ok_span = verify_matched_snippet("short", MatcherBoundaryType::BoundedSpan(10));
        assert!(ok_span.is_ok());
        let err_span = verify_matched_snippet("too long text", MatcherBoundaryType::BoundedSpan(5));
        assert!(err_span.is_err());
    }

    #[test]
    fn test_display_and_helpers_coverage() {
        assert_eq!(MatcherBoundaryType::SingleLineOnly.to_string(), "SingleLineOnly");
        assert_eq!(TimeoutValidationType::PollingLoop.to_string(), "PollingLoop");
        let breach = MatcherViolationType::BoundaryBreach {
            boundary_type: MatcherBoundaryType::WithinTagBound,
            detail: "tag crossed".to_string(),
        };
        assert!(breach.to_string().contains("Boundary breach"));
        let audit_res = audit_matchers_and_timeouts(&[r"^valid$"], &[(100, 500)]);
        assert!(audit_res.is_compliant());
    }
}

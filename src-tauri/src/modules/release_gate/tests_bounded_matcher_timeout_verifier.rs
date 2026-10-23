//! Unit tests for Bounded Matcher & Timeout Verifier.

#[cfg(test)]
mod tests {
    use super::super::bounded_matcher_timeout_verifier::*;

    #[test]
    fn test_bounded_patterns_pass() {
        let valid_patterns = [
            r"^[a-zA-Z0-9_-]+$",
            r"^v\d+\.\d+\.\d+$",
            r"\[v[0-9.]+\]",
            r"fn\s+[a-z_][a-z0-9_]*",
            r"class\s+([A-Za-z0-9_]+)",
            r"#([0-9a-fA-F]{6})",
        ];

        for pattern in valid_patterns {
            assert!(
                verify_matcher_pattern_bounded(pattern).is_ok(),
                "Pattern '{pattern}' should be considered bounded"
            );
        }
    }

    #[test]
    fn test_unbounded_multiline_spans_flagged() {
        let unbounded_patterns = [
            r"(?s).*",
            r"(?s).+",
            r"(?s:.*)",
            r"[\s\S]*",
            r"[\s\S]+",
            r"[\w\W]*",
            r"[\d\D]*",
            r".*",
            r".+",
            r"^.*$",
            r"^.+$",
        ];

        for pattern in unbounded_patterns {
            let res = verify_matcher_pattern_bounded(pattern);
            assert!(
                matches!(res, Err(MatcherBoundaryViolation::UnboundedSpanDetected { .. })),
                "Pattern '{pattern}' must be flagged as UnboundedSpanDetected"
            );
        }
    }

    #[test]
    fn test_valid_timeout_passing() {
        assert!(verify_wait_timeout(100, 1000).is_ok());
        assert!(verify_wait_timeout(5000, 30000).is_ok());
        assert!(verify_wait_timeout(30000, 30000).is_ok());
        assert!(verify_wait_timeout(1, 1).is_ok());
    }

    #[test]
    fn test_zero_timeout_flagged() {
        let res = verify_wait_timeout(0, 30000);
        assert_eq!(
            res,
            Err(MatcherBoundaryViolation::UnboundedWaitTimeout { timeout_ms: 0 })
        );

        let res_zero_cap = verify_wait_timeout(0, 0);
        assert_eq!(
            res_zero_cap,
            Err(MatcherBoundaryViolation::UnboundedWaitTimeout { timeout_ms: 0 })
        );
    }

    #[test]
    fn test_excessive_timeout_flagged() {
        let res = verify_wait_timeout(30001, 30000);
        assert_eq!(
            res,
            Err(MatcherBoundaryViolation::UnboundedWaitTimeout {
                timeout_ms: 30001
            })
        );

        let res_huge = verify_wait_timeout(100_000, 5000);
        assert_eq!(
            res_huge,
            Err(MatcherBoundaryViolation::UnboundedWaitTimeout {
                timeout_ms: 100_000
            })
        );
    }

    #[test]
    fn test_matched_snippet_single_line_boundary() {
        assert!(verify_matched_snippet("hello world", MatcherBoundaryKind::SingleLineOnly).is_ok());

        let res_nl = verify_matched_snippet("hello\nworld", MatcherBoundaryKind::SingleLineOnly);
        assert!(matches!(
            res_nl,
            Err(MatcherBoundaryViolation::CrossesNewlineDisallowed { matched_snippet })
                if matched_snippet == "hello\nworld"
        ));

        let res_cr = verify_matched_snippet("line\rbreak", MatcherBoundaryKind::SingleLineOnly);
        assert!(matches!(
            res_cr,
            Err(MatcherBoundaryViolation::CrossesNewlineDisallowed { matched_snippet })
                if matched_snippet == "line\rbreak"
        ));
    }

    #[test]
    fn test_matched_snippet_within_tag_boundary() {
        assert!(verify_matched_snippet("inner text", MatcherBoundaryKind::WithinTagBound).is_ok());

        let res_tag = verify_matched_snippet("text<div>other", MatcherBoundaryKind::WithinTagBound);
        assert!(matches!(
            res_tag,
            Err(MatcherBoundaryViolation::UnboundedSpanDetected { .. })
        ));

        let res_nl = verify_matched_snippet("tag\ncontent", MatcherBoundaryKind::WithinTagBound);
        assert!(matches!(
            res_nl,
            Err(MatcherBoundaryViolation::CrossesNewlineDisallowed { .. })
        ));
    }

    #[test]
    fn test_matched_snippet_bounded_span() {
        assert!(verify_matched_snippet("short", MatcherBoundaryKind::BoundedSpan(10)).is_ok());

        let res_exceeded =
            verify_matched_snippet("this snippet is too long", MatcherBoundaryKind::BoundedSpan(10));
        assert!(matches!(
            res_exceeded,
            Err(MatcherBoundaryViolation::UnboundedSpanDetected { .. })
        ));
    }

    #[test]
    fn test_display_and_error_formatting() {
        let span_err = MatcherBoundaryViolation::UnboundedSpanDetected {
            pattern: "(?s).*".to_string(),
        };
        assert!(span_err.to_string().contains("Unbounded span detected"));
        assert!(span_err.to_string().contains("(?s).*"));

        let newline_err = MatcherBoundaryViolation::CrossesNewlineDisallowed {
            matched_snippet: "a\nb".to_string(),
        };
        assert!(newline_err.to_string().contains("crossed disallowed newline"));

        let timeout_err = MatcherBoundaryViolation::UnboundedWaitTimeout { timeout_ms: 0 };
        assert!(timeout_err.to_string().contains("Unbounded or invalid wait timeout"));

        use std::error::Error;
        assert!(span_err.source().is_none());
        assert!(newline_err.source().is_none());
        assert!(timeout_err.source().is_none());
    }

    #[test]
    fn test_audit_report_aggregation() {
        let patterns = [r"^[a-z]+$", r"(?s).*"];
        let timeouts = [(5000, 10000), (0, 5000), (20000, 10000)];

        let report = audit_matchers_and_timeouts(&patterns, &timeouts);
        assert!(!report.is_bounded);
        assert!(report.has_violations());
        assert_eq!(report.patterns_audited, 2);
        assert_eq!(report.timeouts_audited, 3);
        assert_eq!(report.violation_count(), 3);
        assert_eq!(report.violations.len(), 3);

        let clean_report = audit_matchers_and_timeouts(&[r"^v\d+$"], &[(1000, 5000)]);
        assert!(clean_report.is_bounded());
        assert!(!clean_report.has_violations());
        assert_eq!(clean_report.violation_count(), 0);

        let default_report = BoundedMatcherAuditReport::default();
        assert!(default_report.is_bounded());
        assert_eq!(default_report.violation_count(), 0);
    }
}

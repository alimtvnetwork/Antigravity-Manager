#[cfg(test)]
mod tests {
    use super::super::bounded_matcher_timeout_guard::{
        apply_bounded_regex_replacement, execute_with_timeout_bounds,
        lint_wait_calls_in_source, poll_until_bounded_timeout, validate_match_boundary,
        validate_replacement_boundaries, validate_wait_timeout, BoundaryCheckType,
        BoundaryViolation, MatchSpan, TimeoutConfig, TimeoutViolation,
    };
    use regex::Regex;

    #[test]
    fn test_single_line_only_match_passes() {
        let text = "let api_key = \"sk-ant-12345\";\nprintln!(\"ready\");";
        let span = MatchSpan::from_first_occurrence(text, "api_key").expect("pattern found");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::SingleLineOnly);
        assert!(result.is_ok(), "Single line token must pass validation");

        let report = result.unwrap();
        assert!(report.is_bounded);
        assert!(!report.spans_line_breaks);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_single_line_only_rejects_line_break_bleed() {
        let text = "first line\nsecond line";
        // Span spanning the newline
        let span = MatchSpan::new(6, 17, "line\nsecond");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::SingleLineOnly);
        assert!(result.is_err());

        match result {
            Err(BoundaryViolation::SpansLineBreaksForbidden {
                line_breaks_count,
                matched_text,
                ..
            }) => {
                assert_eq!(line_breaks_count, 1);
                assert_eq!(matched_text, "line\nsecond");
            }
            other => panic!("Expected SpansLineBreaksForbidden, got {other:?}"),
        }
    }

    #[test]
    fn test_clause_bounded_match_passes_intra_clause() {
        let text = "first clause here; second clause follows.";
        let span = MatchSpan::from_first_occurrence(text, "first clause here").expect("found");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::ClauseBounded);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.is_bounded);
        assert!(!report.crosses_clauses);
    }

    #[test]
    fn test_clause_bounded_rejects_semicolon_crossing() {
        let text = "let a = 1; let b = 2;";
        let span = MatchSpan::new(8, 15, "1; let ");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::ClauseBounded);
        assert!(result.is_err());

        match result {
            Err(BoundaryViolation::CrossesClausesForbidden {
                delimiters,
                matched_text,
                ..
            }) => {
                assert!(delimiters.contains(&';'));
                assert_eq!(matched_text, "1; let ");
            }
            other => panic!("Expected CrossesClausesForbidden, got {other:?}"),
        }
    }

    #[test]
    fn test_tag_enclosed_match_passes_without_tags() {
        let text = "<button class=\"btn-primary\">Submit Order</button>";
        let span = MatchSpan::from_first_occurrence(text, "Submit Order").expect("found");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::TagEnclosed);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.is_bounded);
        assert!(!report.crosses_html_tags);
    }

    #[test]
    fn test_tag_enclosed_rejects_crossing_tags() {
        let text = "<div><span>hello</span></div>";
        let span = MatchSpan::new(2, 13, "ve><span>he");

        let result = validate_match_boundary(text, &span, BoundaryCheckType::TagEnclosed);
        assert!(result.is_err());

        match result {
            Err(BoundaryViolation::CrossesHtmlTagsForbidden { tags, .. }) => {
                assert!(tags.iter().any(|t| t.contains("span")));
            }
            other => panic!("Expected CrossesHtmlTagsForbidden, got {other:?}"),
        }
    }

    #[test]
    fn test_custom_delimiter_boundary_enforcement() {
        let text = "foo|bar|baz";
        let clean_span = MatchSpan::from_first_occurrence(text, "bar").expect("found");
        let bleed_span = MatchSpan::new(2, 8, "o|bar|");

        let ok_res = validate_match_boundary(text, &clean_span, BoundaryCheckType::CustomDelimiter('|'));
        assert!(ok_res.is_ok());

        let err_res = validate_match_boundary(text, &bleed_span, BoundaryCheckType::CustomDelimiter('|'));
        assert!(matches!(
            err_res,
            Err(BoundaryViolation::CrossesCustomDelimiterForbidden { delimiter: '|', .. })
        ));
    }

    #[test]
    fn test_invalid_span_bounds_and_content_mismatch() {
        let text = "hello world";
        let invalid_range = MatchSpan::new(10, 5, "invalid");
        let mismatch = MatchSpan::new(0, 5, "wrong");

        assert!(matches!(
            validate_match_boundary(text, &invalid_range, BoundaryCheckType::SingleLineOnly),
            Err(BoundaryViolation::InvalidSpanRange { .. })
        ));

        assert!(matches!(
            validate_match_boundary(text, &mismatch, BoundaryCheckType::SingleLineOnly),
            Err(BoundaryViolation::SpanContentMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_replacement_boundaries_success() {
        let text = "const secret = 'old_token'; // keep";
        let span = MatchSpan::from_first_occurrence(text, "'old_token'").expect("found");

        let updated = validate_replacement_boundaries(
            text,
            &span,
            "'new_token'",
            BoundaryCheckType::SingleLineOnly,
        )
        .expect("Clean replacement inside bounds should succeed");

        assert_eq!(updated, "const secret = 'new_token'; // keep");
    }

    #[test]
    fn test_validate_replacement_boundaries_rejects_bleed() {
        let text = "const secret = 'old_token';";
        let span = MatchSpan::from_first_occurrence(text, "'old_token'").expect("found");

        // 1. Replacement injects newline into SingleLineOnly
        let err1 = validate_replacement_boundaries(
            text,
            &span,
            "'new_token'\nlet hacked = 1",
            BoundaryCheckType::SingleLineOnly,
        );
        assert!(matches!(err1, Err(BoundaryViolation::SpansLineBreaksForbidden { .. })));

        // 2. Replacement injects HTML tags into TagEnclosed
        let err2 = validate_replacement_boundaries(
            text,
            &span,
            "'new_token' <script>alert(1)</script>",
            BoundaryCheckType::TagEnclosed,
        );
        assert!(matches!(err2, Err(BoundaryViolation::CrossesHtmlTagsForbidden { .. })));
    }

    #[test]
    fn test_apply_bounded_regex_replacement_success() {
        let text = "val_1 = 100; val_2 = 200; val_3 = 300;";
        let regex = Regex::new(r"\d+").expect("valid regex");

        let (result, count) = apply_bounded_regex_replacement(
            text,
            &regex,
            "999",
            BoundaryCheckType::SingleLineOnly,
        )
        .expect("Valid regex replacements should succeed");

        assert_eq!(count, 3);
        assert_eq!(result, "val_1 = 999; val_2 = 999; val_3 = 999;");
    }

    #[test]
    fn test_apply_bounded_regex_replacement_rejects_cross_line_match() {
        let text = "start\nmiddle\nfinish";
        let regex = Regex::new(r"(?s)start.*finish").expect("valid regex");

        let err = apply_bounded_regex_replacement(
            text,
            &regex,
            "replaced",
            BoundaryCheckType::SingleLineOnly,
        );

        assert!(matches!(err, Err(BoundaryViolation::SpansLineBreaksForbidden { .. })));
    }

    #[test]
    fn test_validate_wait_timeout_valid_and_invalid() {
        // 1. Bounded valid timeout
        let valid_config = TimeoutConfig::bounded(5000, 30000, "fetch_profile_data");
        let result = validate_wait_timeout(&valid_config);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.is_bounded);
        assert_eq!(report.effective_timeout_ms, 5000);

        // 2. Unbounded (None) is strictly forbidden
        let unbounded_config = TimeoutConfig::unbounded(30000, "infinite_poll");
        let err_unbounded = validate_wait_timeout(&unbounded_config);
        assert!(matches!(err_unbounded, Err(TimeoutViolation::UnboundedWaitForbidden { .. })));

        // 3. Zero duration is strictly forbidden
        let zero_config = TimeoutConfig::bounded(0, 30000, "instant_poll");
        let err_zero = validate_wait_timeout(&zero_config);
        assert!(matches!(err_zero, Err(TimeoutViolation::UnboundedWaitForbidden { .. })));

        // 4. Exceeds max ceiling
        let over_config = TimeoutConfig::bounded(60000, 30000, "slow_query");
        let err_over = validate_wait_timeout(&over_config);
        assert!(matches!(err_over, Err(TimeoutViolation::ExceedsMaxAllowed { .. })));

        // 5. Empty description
        let empty_config = TimeoutConfig::bounded(5000, 30000, "   ");
        let err_empty = validate_wait_timeout(&empty_config);
        assert!(matches!(err_empty, Err(TimeoutViolation::EmptyOperationDescription)));
    }

    #[tokio::test]
    async fn test_execute_with_timeout_bounds_success() {
        let config = TimeoutConfig::bounded(200, 5000, "fast_computation");
        let result = execute_with_timeout_bounds(&config, async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            100
        })
        .await;

        assert_eq!(result, Ok(100));
    }

    #[tokio::test]
    async fn test_execute_with_timeout_bounds_times_out() {
        let config = TimeoutConfig::bounded(25, 5000, "hanging_operation");
        let result = execute_with_timeout_bounds(&config, async {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            "done"
        })
        .await;

        match result {
            Err(TimeoutViolation::OperationTimedOut {
                operation_description,
                wait_ms,
            }) => {
                assert_eq!(operation_description, "hanging_operation");
                assert_eq!(wait_ms, 25);
            }
            other => panic!("Expected OperationTimedOut, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_poll_until_bounded_timeout_success() {
        let config = TimeoutConfig::bounded(500, 5000, "poll_for_status");
        let mut attempts = 0;

        let result = poll_until_bounded_timeout(&config, 20, || {
            attempts += 1;
            async move {
                if attempts >= 3 {
                    Some(attempts)
                } else {
                    None
                }
            }
        })
        .await;

        assert_eq!(result, Ok(3));
    }

    #[tokio::test]
    async fn test_poll_until_bounded_timeout_times_out() {
        let config = TimeoutConfig::bounded(40, 5000, "unachievable_condition");

        let result: Result<i32, TimeoutViolation> = poll_until_bounded_timeout(
            &config,
            15,
            || async { None },
        )
        .await;

        assert!(matches!(result, Err(TimeoutViolation::OperationTimedOut { .. })));
    }

    #[test]
    fn test_lint_wait_calls_detects_naked_sleeps_and_unbounded_loops() {
        let risky_source = r#"
fn background_task() {
    thread::sleep(Duration::from_secs(1));
}

async fn worker() {
    tokio::time::sleep(Duration::from_millis(100)).await;
}

fn poller() {
    loop {
        thread::sleep(Duration::from_millis(50));
    }
}
"#;

        let violations = lint_wait_calls_in_source(risky_source);
        assert!(!violations.is_empty(), "Risky sleep and loops must be flagged");
        assert!(violations.iter().any(|v| v.contains("thread::sleep")));
        assert!(violations.iter().any(|v| v.contains("tokio::time::sleep")));
        assert!(violations.iter().any(|v| v.contains("Unbounded loop wait detected")));
    }

    #[test]
    fn test_lint_wait_calls_ignores_comments_and_bounded_waits() {
        let safe_source = r#"
// thread::sleep(Duration::from_secs(1));

/*
tokio::time::sleep(Duration::from_millis(100)).await;
*/

async fn bounded_worker() {
    // BoundedTimeout: 5000ms
    thread::sleep(Duration::from_millis(50));
}

async fn safe_poller() {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if check_status() { break; }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await;
}
"#;

        let violations = lint_wait_calls_in_source(safe_source);
        assert!(violations.is_empty(), "Guarded and commented sleeps should pass: {violations:?}");
    }
}

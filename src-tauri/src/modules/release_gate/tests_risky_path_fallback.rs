#[cfg(test)]
mod tests {
    use super::super::risky_path_fallback_guard::{
        execute_with_fallback, inspect_code_for_silent_replacement, lint_replacement_code_pattern,
        validate_replacement, validate_replacement_batch, validate_replacement_strategy,
        FallbackStrategy, PathReplacementExecution, ReplacementValidationReport,
        ReplacementViolation, RiskyPathReplacement, SideEffectOrderGuard,
    };

    #[test]
    fn test_valid_verified_path_restart_strategy() {
        let replacement = RiskyPathReplacement::new(
            "fast_incremental_patcher",
            "patch_in_place",
            "full_process_restart",
            FallbackStrategy::VerifiedPathRestart,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_ok(), "Expected valid report, got: {:?}", result);
        let report = result.unwrap();
        assert!(report.is_valid);
        assert!(report.has_explicit_fallback);
        assert!(report.handles_zero_matches);
        assert!(report.violations.is_empty());
        assert!(report.summary.contains("fast_incremental_patcher"));
    }

    #[test]
    fn test_valid_explicit_handler_strategy() {
        let replacement = RiskyPathReplacement::new(
            "streaming_payload_pipeline",
            "stream_direct_to_socket",
            "buffer_and_verify_payload",
            FallbackStrategy::ExplicitHandler {
                handler_name: "reconnect_and_replay".to_string(),
                target_path: "src-tauri/src/proxy/fallback.rs".to_string(),
            },
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.is_valid);
        assert!(report.has_explicit_fallback);
    }

    #[test]
    fn test_reject_silent_noop_strategy() {
        let replacement = RiskyPathReplacement::new(
            "cached_config_lookup",
            "fetch_from_cache",
            "read_from_disk",
            FallbackStrategy::SilentNoOp,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        match violation {
            ReplacementViolation::SilentNoOpForbidden { context } => {
                assert!(context.contains("cached_config_lookup"));
            }
            other => panic!("Expected SilentNoOpForbidden, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_unspecified_fallback_strategy() {
        let replacement = RiskyPathReplacement::new(
            "speculative_token_route",
            "eval_speculative_budget",
            "eval_static_budget",
            FallbackStrategy::Unspecified,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        match violation {
            ReplacementViolation::MissingFallbackStrategy { replacement_name } => {
                assert_eq!(replacement_name, "speculative_token_route");
            }
            other => panic!("Expected MissingFallbackStrategy, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_missing_zero_match_handling() {
        let mut replacement = RiskyPathReplacement::new(
            "regex_fast_path",
            "search_fast_regex",
            "search_linear_scan",
            FallbackStrategy::VerifiedPathRestart,
        );
        replacement.handles_zero_matches = false;

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReplacementViolation::MissingZeroMatchHandling { replacement_name } => {
                assert_eq!(replacement_name, "regex_fast_path");
            }
            other => panic!("Expected MissingZeroMatchHandling, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_empty_speculative_path() {
        let replacement = RiskyPathReplacement::new(
            "test_replacement",
            "   ",
            "verified_path",
            FallbackStrategy::VerifiedPathRestart,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReplacementViolation::EmptySpeculativePath { replacement_name } => {
                assert_eq!(replacement_name, "test_replacement");
            }
            other => panic!("Expected EmptySpeculativePath, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_empty_verified_path() {
        let replacement = RiskyPathReplacement::new(
            "test_replacement",
            "speculative_path",
            "",
            FallbackStrategy::VerifiedPathRestart,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReplacementViolation::EmptyVerifiedPath { replacement_name } => {
                assert_eq!(replacement_name, "test_replacement");
            }
            other => panic!("Expected EmptyVerifiedPath, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_identical_speculative_and_fallback_path() {
        let replacement = RiskyPathReplacement::new(
            "redundant_op",
            "same/path/file.rs",
            "same/path/file.rs",
            FallbackStrategy::VerifiedPathRestart,
        );

        let result = validate_replacement_strategy(&replacement);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReplacementViolation::RedundantFallbackSamePath { path } => {
                assert_eq!(path, "same/path/file.rs");
            }
            other => panic!("Expected RedundantFallbackSamePath, got {:?}", other),
        }
    }

    #[test]
    fn test_lint_replacement_code_pattern_detects_silent_noop() {
        let bad_code = r#"
            if matches.is_empty() {
                return Ok(());
            }
            execute_optimized_step();
        "#;

        let violations = lint_replacement_code_pattern(bad_code);
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            ReplacementViolation::SilentNoOpForbidden { context } => {
                assert!(context.contains("matches.is_empty()"));
            }
            other => panic!("Expected SilentNoOpForbidden, got {:?}", other),
        }
    }

    #[test]
    fn test_lint_replacement_code_pattern_clean_code() {
        let clean_code = r#"
            if matches.is_empty() {
                return trigger_full_restart_fallback();
            }
            execute_optimized_step();
        "#;

        let violations = lint_replacement_code_pattern(clean_code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_path_replacement_execution_validation_happy_path() {
        let exec = PathReplacementExecution::new(
            "replace_router_path",
            "legacy fast-path router",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_matches(5);

        assert!(validate_replacement(&exec).is_ok());
    }

    #[test]
    fn test_path_replacement_execution_silent_noop_rejected() {
        let exec = PathReplacementExecution::new(
            "replace_dns",
            "custom resolver",
            FallbackStrategy::SilentNoOp,
        )
        .with_matches(1);

        assert!(matches!(
            validate_replacement(&exec),
            Err(ReplacementViolation::SilentNoOpForbidden { .. })
        ));
    }

    #[test]
    fn test_path_replacement_execution_unspecified_rejected() {
        let exec = PathReplacementExecution::new(
            "replace_port",
            "port scanner",
            FallbackStrategy::Unspecified,
        );

        assert!(matches!(
            validate_replacement(&exec),
            Err(ReplacementViolation::MissingExplicitFallback { .. })
        ));
    }

    #[test]
    fn test_path_replacement_execution_unvalidated_side_effects_rejected() {
        let exec = PathReplacementExecution::new(
            "replace_writer",
            "atomic temp file sync",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_prevalidation(false);

        assert!(matches!(
            validate_replacement(&exec),
            Err(ReplacementViolation::SideEffectsNotPrevalidated { .. })
        ));
    }

    #[test]
    fn test_path_replacement_execution_unbounded_wait_rejected() {
        let exec = PathReplacementExecution::new(
            "replace_socket",
            "async wait",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_timeout_ms(None);

        assert!(matches!(
            validate_replacement(&exec),
            Err(ReplacementViolation::UnboundedWaitMissingTimeout { .. })
        ));
    }

    #[test]
    fn test_path_replacement_execution_overbroad_scope_rejected() {
        let exec = PathReplacementExecution::new(
            "replace_config",
            "wildcard replacement",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_narrow_scope(false);

        assert!(matches!(
            validate_replacement(&exec),
            Err(ReplacementViolation::OverbroadScopeDetected { .. })
        ));
    }

    #[test]
    fn test_validate_replacement_batch() {
        let valid = PathReplacementExecution::new(
            "op_1",
            "target 1",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_matches(2);
        let invalid =
            PathReplacementExecution::new("op_2", "target 2", FallbackStrategy::SilentNoOp);

        let violations = validate_replacement_batch(&[valid, invalid]);
        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0],
            ReplacementViolation::SilentNoOpForbidden { .. }
        ));
    }

    #[test]
    fn test_execute_with_fallback_success_path() {
        let execution = PathReplacementExecution::new(
            "replace_compression",
            "gzip accelerator",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_matches(1);

        let outcome = execute_with_fallback(
            &execution,
            || Ok(Some("compressed_bytes")),
            || Ok("fallback_bytes"),
        );

        assert_eq!(outcome.unwrap(), "compressed_bytes");
    }

    #[test]
    fn test_execute_with_fallback_zero_matches_triggers_verified_fallback() {
        let execution = PathReplacementExecution::new(
            "replace_string_parser",
            "regex matcher",
            FallbackStrategy::VerifiedPathRestart,
        )
        .with_matches(0);

        let outcome = execute_with_fallback(
            &execution,
            || Ok(Some("candidate_result")),
            || Ok("verified_baseline_parser_result"),
        );

        assert_eq!(outcome.unwrap(), "verified_baseline_parser_result");
    }

    #[test]
    fn test_execute_with_fallback_failure_triggers_fallback() {
        let execution = PathReplacementExecution::new(
            "replace_parser",
            "token stream",
            FallbackStrategy::ExplicitHandler {
                handler_name: "std_parser".to_string(),
                target_path: "parser.rs".to_string(),
            },
        )
        .with_matches(1);

        let outcome = execute_with_fallback(
            &execution,
            || Err("stream corrupted".to_string()),
            || Ok("std_parser_result"),
        );

        assert_eq!(outcome.unwrap(), "std_parser_result");
    }

    #[test]
    fn test_side_effect_order_guard_happy_path() {
        let mut guard = SideEffectOrderGuard::new();
        guard.mark_validated();
        assert!(guard.mark_credentials_written().is_ok());
        assert!(guard.execute_destructive_action().is_ok());
        assert!(guard.point_of_no_return_executed);
    }

    #[test]
    fn test_side_effect_order_guard_kill_before_credentials_rejected() {
        let mut guard = SideEffectOrderGuard::new();
        guard.mark_validated();
        let err = guard.execute_destructive_action().unwrap_err();
        assert!(matches!(
            err,
            ReplacementViolation::SideEffectsNotPrevalidated { .. }
        ));
    }

    #[test]
    fn test_side_effect_order_guard_write_credentials_before_validation_rejected() {
        let mut guard = SideEffectOrderGuard::new();
        let err = guard.mark_credentials_written().unwrap_err();
        assert!(matches!(
            err,
            ReplacementViolation::SideEffectsNotPrevalidated { .. }
        ));
    }

    #[test]
    fn test_inspect_code_for_silent_replacement() {
        let bad_code = r#"
            // replace fast path
            if matches == 0 {}
            // silent no-op replacement fallback
            kill_process();
        "#;

        let violations = inspect_code_for_silent_replacement(bad_code);
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| matches!(v, ReplacementViolation::SilentNoOpForbidden { .. })));
        assert!(violations
            .iter()
            .any(|v| matches!(v, ReplacementViolation::UnhandledZeroMatches { .. })));
        assert!(violations
            .iter()
            .any(|v| matches!(v, ReplacementViolation::SideEffectsNotPrevalidated { .. })));
    }
}

//! Unit tests for Side Effect Order Enforcer & Risky Path Replacement Guard (`Imp-655`).

#[cfg(test)]
mod tests {
    use crate::modules::instance::side_effect_order_enforcer::{
        validate_side_effect_order, validate_side_effect_order_with_policy,
        SideEffectExecutionTracker, SideEffectKind, SideEffectOrderPolicy,
        SideEffectOrderViolation, SideEffectPlan, SideEffectStage, DEFAULT_MAX_STAGE_TIMEOUT_MS,
    };

    #[test]
    fn test_empty_plan_fails_validation() {
        let plan = SideEffectPlan::new("empty_test");
        let result = validate_side_effect_order(&plan);
        assert_eq!(result, Err(SideEffectOrderViolation::EmptyPlan));
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn test_valid_order_complete_lifecycle() {
        let plan = SideEffectPlan::new("valid_lifecycle")
            .with_stage(SideEffectStage::validation(
                "val_01",
                "Verify directory structure and permissions",
            ))
            .with_stage(SideEffectStage::credential_persist(
                "cred_01",
                "Write OAuth credentials before stopping runtime",
            ))
            .with_stage(SideEffectStage::process_termination(
                "kill_01",
                "Gracefully terminate target IDE process",
                5000,
            ))
            .with_stage(SideEffectStage::storage_deletion(
                "del_01",
                "Clean temporary socket and cache data",
            ));

        assert_eq!(plan.len(), 4);
        assert_eq!(validate_side_effect_order(&plan), Ok(()));
        assert_eq!(plan.validate(), Ok(()));
    }

    #[test]
    fn test_valid_order_credentials_before_kill() {
        let plan = SideEffectPlan::new("cred_then_kill")
            .with_stage(SideEffectStage::credential_persist(
                "cred_persist",
                "Persist user credentials to secure storage",
            ))
            .with_stage(SideEffectStage::process_termination(
                "proc_kill",
                "Terminate previous instance process",
                3000,
            ));

        assert_eq!(validate_side_effect_order(&plan), Ok(()));
    }

    #[test]
    fn test_valid_order_validation_before_deletion() {
        let plan = SideEffectPlan::new("val_then_del")
            .with_stage(SideEffectStage::validation(
                "validate_path",
                "Validate target directory path is within workspace bounds",
            ))
            .with_stage(SideEffectStage::storage_deletion(
                "delete_dir",
                "Remove isolated temporary sandbox folder",
            ));

        assert_eq!(validate_side_effect_order(&plan), Ok(()));
    }

    #[test]
    fn test_inverted_order_kill_before_write_credentials() {
        // AGENTS.md invariant: "write credentials before killing a process"
        // Inverted: ProcessTermination scheduled before CredentialPersist
        let plan = SideEffectPlan::new("inverted_kill_before_cred")
            .with_stage(SideEffectStage::process_termination(
                "kill_stage",
                "Kill active instance process",
                4000,
            ))
            .with_stage(SideEffectStage::credential_persist(
                "cred_stage",
                "Attempt to persist credentials after process is killed",
            ));

        let result = validate_side_effect_order(&plan);
        assert_eq!(
            result,
            Err(SideEffectOrderViolation::InvertedCredentialKillOrder {
                kill_stage_id: "kill_stage".to_string(),
                credential_stage_id: "cred_stage".to_string(),
                kill_index: 0,
                credential_index: 1,
            })
        );
    }

    #[test]
    fn test_inverted_order_multiple_stages_kill_before_credentials() {
        let plan = SideEffectPlan::new("multi_stage_inverted")
            .with_stage(SideEffectStage::validation(
                "val_01",
                "Pre-execution validation",
            ))
            .with_stage(SideEffectStage::process_termination(
                "kill_01",
                "Terminate process first",
                2500,
            ))
            .with_stage(SideEffectStage::credential_persist(
                "cred_01",
                "Late credential flush",
            ));

        let result = validate_side_effect_order(&plan);
        match result {
            Err(SideEffectOrderViolation::InvertedCredentialKillOrder {
                kill_stage_id,
                credential_stage_id,
                kill_index,
                credential_index,
            }) => {
                assert_eq!(kill_stage_id, "kill_01");
                assert_eq!(credential_stage_id, "cred_01");
                assert_eq!(kill_index, 1);
                assert_eq!(credential_index, 2);
            }
            other => panic!("Expected InvertedCredentialKillOrder, got {:?}", other),
        }
    }

    #[test]
    fn test_inverted_order_delete_before_validation() {
        // AGENTS.md invariant: "validate before deleting"
        // Inverted: StorageDeletion scheduled before Validation
        let plan = SideEffectPlan::new("inverted_del_val")
            .with_stage(SideEffectStage::storage_deletion(
                "del_stage",
                "Premature deletion of profile cache",
            ))
            .with_stage(SideEffectStage::validation(
                "val_stage",
                "Late validation check",
            ));

        let result = validate_side_effect_order(&plan);
        assert_eq!(
            result,
            Err(SideEffectOrderViolation::InvertedValidationDeletionOrder {
                deletion_stage_id: "del_stage".to_string(),
                validation_stage_id: "val_stage".to_string(),
                deletion_index: 0,
                validation_index: 1,
            })
        );
    }

    #[test]
    fn test_missing_validation_before_deletion() {
        let plan = SideEffectPlan::new("missing_val_for_del").with_stage(
            SideEffectStage::storage_deletion("unvalidated_del", "Delete storage with no checks"),
        );

        let result = validate_side_effect_order(&plan);
        assert_eq!(
            result,
            Err(SideEffectOrderViolation::MissingValidationBeforeDeletion {
                deletion_stage_id: "unvalidated_del".to_string(),
                deletion_index: 0,
            })
        );
    }

    #[test]
    fn test_timeout_bounds_unbounded_none_fails() {
        // AGENTS.md invariant: "Bound every wait with a timeout."
        let mut stage = SideEffectStage::new("unbounded_kill", SideEffectKind::ProcessTermination);
        stage.timeout_ms = None;

        let plan = SideEffectPlan::new("unbounded_plan").with_stage(stage);
        let result = validate_side_effect_order(&plan);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::UnboundedWaitTimeout {
                stage_id: "unbounded_kill".to_string(),
                kind: SideEffectKind::ProcessTermination,
            })
        );
    }

    #[test]
    fn test_timeout_bounds_unbounded_zero_fails() {
        let stage =
            SideEffectStage::process_termination("zero_timeout_kill", "Kill with zero timeout", 0);

        let plan = SideEffectPlan::new("zero_timeout_plan").with_stage(stage);
        let result = validate_side_effect_order(&plan);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::UnboundedWaitTimeout {
                stage_id: "zero_timeout_kill".to_string(),
                kind: SideEffectKind::ProcessTermination,
            })
        );
    }

    #[test]
    fn test_timeout_bounds_exceeds_maximum_fails() {
        let max_limit = 10_000;
        let policy = SideEffectOrderPolicy::default().with_max_timeout_ms(max_limit);

        let stage = SideEffectStage::process_termination(
            "excessive_timeout_kill",
            "Kill with overly large timeout",
            15_000,
        );

        let plan = SideEffectPlan::new("excessive_timeout_plan")
            .with_policy(policy)
            .with_stage(stage);

        let result = validate_side_effect_order(&plan);
        assert_eq!(
            result,
            Err(SideEffectOrderViolation::TimeoutExceedsMaximum {
                stage_id: "excessive_timeout_kill".to_string(),
                timeout_ms: 15_000,
                max_timeout_ms: max_limit,
            })
        );
    }

    #[test]
    fn test_timeout_bounds_exact_maximum_allowed() {
        let stage = SideEffectStage::process_termination(
            "boundary_kill",
            "Kill with exact default maximum timeout",
            DEFAULT_MAX_STAGE_TIMEOUT_MS,
        );

        let plan = SideEffectPlan::new("boundary_plan").with_stage(stage);
        assert_eq!(validate_side_effect_order(&plan), Ok(()));
    }

    #[test]
    fn test_replacement_missing_fallback_fails() {
        // AGENTS.md: "Keep an explicit fallback to the verified path whenever you replace it"
        let stage = SideEffectStage::validation("fast_heuristic", "Fast path heuristic check")
            .with_replacement(true);

        let plan = SideEffectPlan::new("replacement_no_fallback").with_stage(stage);
        let result = validate_side_effect_order(&plan);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::MissingVerifiedPathFallback {
                stage_id: "fast_heuristic".to_string(),
                operation: "Fast path heuristic check".to_string(),
            })
        );
    }

    #[test]
    fn test_replacement_empty_string_fallback_fails() {
        let stage = SideEffectStage::validation("fast_heuristic", "Fast path heuristic check")
            .with_replacement(true)
            .with_fallback("   ");

        let plan = SideEffectPlan::new("replacement_empty_fallback").with_stage(stage);
        let result = validate_side_effect_order(&plan);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::MissingVerifiedPathFallback {
                stage_id: "fast_heuristic".to_string(),
                operation: "Fast path heuristic check".to_string(),
            })
        );
    }

    #[test]
    fn test_replacement_with_valid_fallback_succeeds() {
        let stage = SideEffectStage::validation("fast_heuristic", "Fast path heuristic check")
            .with_replacement(true)
            .with_fallback("verified_full_restart");

        assert!(stage.has_fallback());
        let plan = SideEffectPlan::new("replacement_with_fallback").with_stage(stage);
        assert_eq!(validate_side_effect_order(&plan), Ok(()));
    }

    #[test]
    fn test_side_effect_kind_predicates_and_display() {
        assert!(!SideEffectKind::Validation.is_irreversible());
        assert!(SideEffectKind::Validation.is_safe_precursor());
        assert!(SideEffectKind::Validation.is_validation());
        assert_eq!(SideEffectKind::Validation.as_str(), "validation");
        assert_eq!(format!("{}", SideEffectKind::Validation), "validation");

        assert!(!SideEffectKind::CredentialPersist.is_irreversible());
        assert!(SideEffectKind::CredentialPersist.is_safe_precursor());
        assert!(SideEffectKind::CredentialPersist.is_credential_persist());
        assert_eq!(
            SideEffectKind::CredentialPersist.as_str(),
            "credential_persist"
        );
        assert_eq!(
            format!("{}", SideEffectKind::CredentialPersist),
            "credential_persist"
        );

        assert!(SideEffectKind::ProcessTermination.is_irreversible());
        assert!(!SideEffectKind::ProcessTermination.is_safe_precursor());
        assert!(SideEffectKind::ProcessTermination.is_process_termination());
        assert_eq!(
            SideEffectKind::ProcessTermination.as_str(),
            "process_termination"
        );
        assert_eq!(
            format!("{}", SideEffectKind::ProcessTermination),
            "process_termination"
        );

        assert!(SideEffectKind::StorageDeletion.is_irreversible());
        assert!(!SideEffectKind::StorageDeletion.is_safe_precursor());
        assert!(SideEffectKind::StorageDeletion.is_storage_deletion());
        assert_eq!(SideEffectKind::StorageDeletion.as_str(), "storage_deletion");
        assert_eq!(
            format!("{}", SideEffectKind::StorageDeletion),
            "storage_deletion"
        );
    }

    #[test]
    fn test_stage_builder_methods() {
        let stage = SideEffectStage::new("stage_a", SideEffectKind::Validation)
            .with_description("Custom description")
            .with_timeout_ms(1200)
            .with_fallback("fallback_path")
            .with_replacement(true);

        assert_eq!(stage.id, "stage_a");
        assert_eq!(stage.description, "Custom description");
        assert_eq!(stage.timeout_ms, Some(1200));
        assert_eq!(stage.fallback.as_deref(), Some("fallback_path"));
        assert!(stage.is_replacement);
        assert!(stage.has_timeout());
        assert!(stage.has_fallback());
        assert!(!stage.is_irreversible());
    }

    #[test]
    fn test_policy_builder_methods() {
        let policy = SideEffectOrderPolicy::new()
            .with_max_timeout_ms(60_000)
            .with_require_bounded_timeout(false)
            .with_require_fallback_on_replacement(false);

        assert_eq!(policy.max_timeout_ms, 60_000);
        assert!(!policy.require_bounded_timeout);
        assert!(!policy.require_fallback_on_replacement);

        // When require_bounded_timeout is false, unbounded termination stage passes
        let mut stage = SideEffectStage::new("unbounded_term", SideEffectKind::ProcessTermination);
        stage.timeout_ms = None;
        let plan = SideEffectPlan::new("test_policy").with_stage(stage);

        let result = validate_side_effect_order_with_policy(&plan, &policy);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_runtime_execution_tracker_success() {
        let mut tracker = SideEffectExecutionTracker::new();
        assert_eq!(tracker.count(), 0);

        let stage1 = SideEffectStage::validation("v1", "Initial validation");
        assert_eq!(tracker.record_executed(stage1), Ok(()));
        assert_eq!(tracker.count(), 1);

        let stage2 = SideEffectStage::credential_persist("c1", "Persist auth token");
        assert_eq!(tracker.record_executed(stage2), Ok(()));
        assert_eq!(tracker.count(), 2);

        let stage3 = SideEffectStage::process_termination("k1", "Kill old process", 4000);
        assert_eq!(tracker.record_executed(stage3), Ok(()));
        assert_eq!(tracker.count(), 3);

        let stage4 = SideEffectStage::storage_deletion("d1", "Delete old cache");
        assert_eq!(tracker.record_executed(stage4), Ok(()));
        assert_eq!(tracker.count(), 4);
        assert_eq!(tracker.executed_stages().len(), 4);
    }

    #[test]
    fn test_runtime_execution_tracker_rejects_kill_before_credentials() {
        let mut tracker = SideEffectExecutionTracker::new();

        let stage1 = SideEffectStage::process_termination("k1", "Kill process first", 5000);
        assert_eq!(tracker.record_executed(stage1), Ok(()));

        let stage2 = SideEffectStage::credential_persist("c1", "Try to write credentials late");
        let result = tracker.record_executed(stage2);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::InvertedCredentialKillOrder {
                kill_stage_id: "k1".to_string(),
                credential_stage_id: "c1".to_string(),
                kill_index: 0,
                credential_index: 1,
            })
        );
        // Tracker state remains at 1, preventing the invalid stage from being committed
        assert_eq!(tracker.count(), 1);
    }

    #[test]
    fn test_runtime_execution_tracker_rejects_delete_without_validation() {
        let mut tracker = SideEffectExecutionTracker::new();

        let del_stage = SideEffectStage::storage_deletion("d1", "Premature deletion");
        let result = tracker.record_executed(del_stage);

        assert_eq!(
            result,
            Err(SideEffectOrderViolation::MissingValidationBeforeDeletion {
                deletion_stage_id: "d1".to_string(),
                deletion_index: 0,
            })
        );
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_error_display_formatting() {
        let err1 = SideEffectOrderViolation::EmptyPlan;
        assert_eq!(format!("{}", err1), "Side effect plan is empty");

        let err2 = SideEffectOrderViolation::InvertedCredentialKillOrder {
            kill_stage_id: "k_0".to_string(),
            credential_stage_id: "c_1".to_string(),
            kill_index: 0,
            credential_index: 1,
        };
        assert!(format!("{}", err2).contains("k_0"));
        assert!(format!("{}", err2).contains("c_1"));

        let err3 = SideEffectOrderViolation::MissingValidationBeforeDeletion {
            deletion_stage_id: "del_0".to_string(),
            deletion_index: 0,
        };
        assert!(format!("{}", err3).contains("del_0"));

        let err4 = SideEffectOrderViolation::InvertedValidationDeletionOrder {
            deletion_stage_id: "del_0".to_string(),
            validation_stage_id: "val_1".to_string(),
            deletion_index: 0,
            validation_index: 1,
        };
        assert!(format!("{}", err4).contains("del_0"));
        assert!(format!("{}", err4).contains("val_1"));

        let err5 = SideEffectOrderViolation::UnboundedWaitTimeout {
            stage_id: "wait_0".to_string(),
            kind: SideEffectKind::ProcessTermination,
        };
        assert!(format!("{}", err5).contains("wait_0"));

        let err6 = SideEffectOrderViolation::TimeoutExceedsMaximum {
            stage_id: "wait_0".to_string(),
            timeout_ms: 50_000,
            max_timeout_ms: 10_000,
        };
        assert!(format!("{}", err6).contains("50000ms"));

        let err7 = SideEffectOrderViolation::MissingVerifiedPathFallback {
            stage_id: "repl_0".to_string(),
            operation: "risky op".to_string(),
        };
        assert!(format!("{}", err7).contains("repl_0"));
    }
}

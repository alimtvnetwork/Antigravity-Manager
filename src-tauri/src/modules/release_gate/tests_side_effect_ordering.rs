#[cfg(test)]
mod tests {
    use super::super::side_effect_ordering_guard::{
        audit_operation_pipeline, inspect_operation_pipeline, is_irreversible_keyword,
        is_persistence_keyword, validate_operation_pipeline, validate_operation_sequence,
        OperationCategory, OperationPhase, OperationPipelineBuilder, OperationStep,
        OrderingValidationReport, OrderingViolation, PlannedOperation,
        SideEffectOrderingViolation,
    };

    // ========================================================================
    // Pipeline Model Tests (Task 2 Invariant Requirements)
    // ========================================================================

    #[test]
    fn test_empty_pipeline_fails() {
        let empty: Vec<OperationStep> = vec![];
        let result = validate_operation_pipeline(&empty);
        assert_eq!(result, Err(SideEffectOrderingViolation::EmptyPipeline));

        let report = audit_operation_pipeline(&empty);
        assert!(!report.is_valid);
        assert_eq!(report.violations.len(), 1);
    }

    #[test]
    fn test_valid_pipeline_succeeds() {
        let steps = OperationPipelineBuilder::new()
            .validate("validate_credentials_format", "Check token syntax and permissions")
            .persist("write_credentials", "Persist auth token to encrypted keystore")
            .execute_irreversible("terminate_old_daemon", "Kill existing background process")
            .cleanup("delete_staging_files", "Remove transient staging files")
            .build();

        let result = validate_operation_pipeline(&steps);
        assert!(result.is_ok());

        let report = inspect_operation_pipeline(&steps);
        assert!(report.is_valid);
        assert_eq!(report.total_steps, 4);
        assert!(report.has_irreversible_execution);
        assert_eq!(report.point_of_no_return_index, Some(2));
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_write_credentials_after_killing_process_is_rejected() {
        // AGENTS.md Invariant: "write credentials before killing a process"
        // Here, killing the process occurs at index 1, but writing credentials occurs at index 2.
        let steps = vec![
            OperationStep::validation("validate_inputs", "Verify configuration arguments"),
            OperationStep::irreversible("kill_active_process", "Kill the active daemon process"),
            OperationStep::persistence("write_credentials", "Write updated credentials to disk"),
            OperationStep::cleanup("cleanup_staging", "Clean up temporary directory"),
        ];

        let result = validate_operation_pipeline(&steps);
        match result {
            Err(SideEffectOrderingViolation::IrreversibleBeforePersistence {
                irreversible_step,
                persistence_step,
                details,
            }) => {
                assert_eq!(irreversible_step, "kill_active_process");
                assert_eq!(persistence_step, "write_credentials");
                assert!(details.contains("write credentials before killing a process"));
            }
            other => panic!("Expected IrreversibleBeforePersistence, got: {:?}", other),
        }
    }

    #[test]
    fn test_delete_before_validate_is_rejected() {
        // AGENTS.md Invariant: "validate before deleting"
        // Here, deleting the database occurs before validating integrity.
        let steps = vec![
            OperationStep::persistence("backup_database", "Create snapshot backup"),
            OperationStep::irreversible("delete_database", "Drop active SQLite database"),
            OperationStep::validation("validate_syntax", "Check schema syntax"),
        ];

        let result = validate_operation_pipeline(&steps);
        match result {
            Err(SideEffectOrderingViolation::IrreversibleBeforeValidation {
                irreversible_step,
                validation_step,
                details,
            }) => {
                assert_eq!(irreversible_step, "delete_database");
                assert_eq!(validation_step, "validate_syntax");
                assert!(details.contains("validate before deleting"));
            }
            other => panic!("Expected IrreversibleBeforeValidation, got: {:?}", other),
        }
    }

    #[test]
    fn test_missing_validation_before_destruction() {
        let steps = vec![
            OperationStep::irreversible("truncate_table", "Truncate audit logs table"),
            OperationStep::cleanup("log_telemetry", "Record telemetry event"),
        ];

        let result = validate_operation_pipeline(&steps);
        match result {
            Err(SideEffectOrderingViolation::MissingValidationBeforeDestruction {
                irreversible_step,
                details,
            }) => {
                assert_eq!(irreversible_step, "truncate_table");
                assert!(details.contains("preceding Validation step"));
            }
            other => panic!("Expected MissingValidationBeforeDestruction, got: {:?}", other),
        }
    }

    #[test]
    fn test_phase_order_regression_detection() {
        // Cleanup phase precedes Staging phase
        let steps = vec![
            OperationStep::validation("check_env", "Check environment variables"),
            OperationStep::cleanup("clean_cache", "Purge cache directory"),
            OperationStep::staging("prepare_bundle", "Stage release files"),
        ];

        let report = audit_operation_pipeline(&steps);
        assert!(!report.is_valid);
        assert!(report
            .violations
            .iter()
            .any(|v| matches!(v, SideEffectOrderingViolation::PhaseOrderRegression { .. })));
    }

    #[test]
    fn test_heuristic_keyword_detection() {
        assert!(is_persistence_keyword("save_auth_token", "store credentials"));
        assert!(is_persistence_keyword("write_credentials_file", ""));
        assert!(is_persistence_keyword("backup_config", "backup before switch"));
        assert!(!is_persistence_keyword("check_syntax", "parse json input"));

        assert!(is_irreversible_keyword("kill_instance", "terminate running IDE"));
        assert!(is_irreversible_keyword("delete_old_cache", "remove files"));
        assert!(is_irreversible_keyword("drop_table", "drop SQLite table"));
        assert!(!is_irreversible_keyword("verify_path", "check if folder exists"));
    }

    #[test]
    fn test_operation_phase_properties() {
        assert_eq!(OperationPhase::Validation.rank(), 1);
        assert_eq!(OperationPhase::Staging.rank(), 2);
        assert_eq!(OperationPhase::IrreversibleExecution.rank(), 3);
        assert_eq!(OperationPhase::Cleanup.rank(), 4);

        assert!(OperationPhase::Validation.is_pre_execution());
        assert!(OperationPhase::Staging.is_pre_execution());
        assert!(!OperationPhase::IrreversibleExecution.is_pre_execution());
        assert!(OperationPhase::IrreversibleExecution.is_point_of_no_return());
        assert!(OperationPhase::Cleanup.is_cleanup());

        assert_eq!(format!("{}", OperationPhase::Validation), "Validation");
        assert_eq!(format!("{}", OperationPhase::Staging), "Staging");
        assert_eq!(
            format!("{}", OperationPhase::IrreversibleExecution),
            "IrreversibleExecution"
        );
        assert_eq!(format!("{}", OperationPhase::Cleanup), "Cleanup");
    }

    #[test]
    fn test_builder_pipeline_audit() {
        let builder = OperationPipelineBuilder::new()
            .validate("val1", "Validate schema")
            .stage("stg1", "Prepare working buffer")
            .persist("persist1", "Save credentials")
            .execute_irreversible("exec1", "Commit database migration")
            .cleanup("cln1", "Remove temporary artifacts");

        let report = builder.audit_pipeline();
        assert!(report.is_valid);
        assert_eq!(report.total_steps, 5);
        assert!(builder.validate_pipeline().is_ok());
    }

    // ========================================================================
    // Planned Operation & Sequence Model Tests (Alternative Schema Support)
    // ========================================================================

    #[test]
    fn test_valid_operation_sequence_with_safe_ordering() {
        let ops = vec![
            PlannedOperation::new("1", "validate_configuration", OperationCategory::PreValidation),
            PlannedOperation::new("2", "write_credentials_file", OperationCategory::CredentialPersistence),
            PlannedOperation::new("3", "snapshot_state", OperationCategory::StateBackup),
            PlannedOperation::new("4", "boundary_commit", OperationCategory::PointOfNoReturnBoundary),
            PlannedOperation::new("5", "terminate_old_process", OperationCategory::ProcessKill),
            PlannedOperation::new("6", "cleanup_stale_cache", OperationCategory::FilesystemPurge),
        ];

        let result = validate_operation_sequence(&ops);
        assert!(result.is_ok(), "Expected valid sequence, got: {:?}", result);
        let report = result.unwrap();
        assert!(report.is_order_compliant);
        assert_eq!(report.point_of_no_return_index, Some(3));
        assert_eq!(report.pre_boundary_ops_count, 3);
        assert_eq!(report.post_boundary_ops_count, 3);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_reject_process_kill_before_credential_write() {
        // AGENTS.md rule: "write credentials before killing a process"
        let ops = vec![
            PlannedOperation::new("1", "validate_input", OperationCategory::PreValidation),
            PlannedOperation::new("2", "kill_existing_server", OperationCategory::ProcessKill),
            PlannedOperation::new("3", "save_new_credentials", OperationCategory::CredentialPersistence),
        ];

        let result = validate_operation_sequence(&ops);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        match violation {
            OrderingViolation::ProcessKillBeforeCredentialWrite { kill_index, write_index } => {
                assert_eq!(kill_index, 1);
                assert_eq!(write_index, 2);
            }
            other => panic!("Expected ProcessKillBeforeCredentialWrite, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_deletion_before_validation() {
        // AGENTS.md rule: "validate before deleting"
        let ops = vec![
            PlannedOperation::new("1", "delete_active_profile", OperationCategory::EntityDeletion),
            PlannedOperation::new("2", "validate_profile_schema", OperationCategory::PreValidation),
        ];

        let result = validate_operation_sequence(&ops);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        match violation {
            OrderingViolation::DestructiveActionBeforePreValidation { action_name, action_index } => {
                assert_eq!(action_name, "delete_active_profile");
                assert_eq!(action_index, 0);
            }
            OrderingViolation::DeletionBeforeValidation { delete_index, validate_index } => {
                assert_eq!(delete_index, 0);
                assert_eq!(validate_index, 1);
            }
            other => panic!("Expected DestructiveActionBeforePreValidation or DeletionBeforeValidation, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_destructive_op_with_no_prior_validation() {
        let ops = vec![
            PlannedOperation::new("1", "wipe_cache_dir", OperationCategory::FilesystemPurge),
        ];

        let result = validate_operation_sequence(&ops);
        assert!(result.is_err());
        match result.unwrap_err() {
            OrderingViolation::MissingValidationBeforeIrreversibleOp { op_name } => {
                assert_eq!(op_name, "wipe_cache_dir");
            }
            other => panic!("Expected MissingValidationBeforeIrreversibleOp, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_empty_sequence() {
        let result = validate_operation_sequence(&[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            OrderingViolation::EmptyOperationSequence => {}
            other => panic!("Expected EmptyOperationSequence, got {:?}", other),
        }
    }

    #[test]
    fn test_operation_category_classification_methods() {
        assert!(OperationCategory::ProcessKill.is_irreversible());
        assert!(OperationCategory::EntityDeletion.is_irreversible());
        assert!(OperationCategory::FilesystemPurge.is_irreversible());
        assert!(!OperationCategory::PreValidation.is_irreversible());
        assert!(!OperationCategory::CredentialPersistence.is_irreversible());

        assert!(OperationCategory::PreValidation.is_safety_prerequisite());
        assert!(OperationCategory::CredentialPersistence.is_safety_prerequisite());
        assert!(OperationCategory::StateBackup.is_safety_prerequisite());
        assert!(!OperationCategory::ProcessKill.is_safety_prerequisite());
    }
}

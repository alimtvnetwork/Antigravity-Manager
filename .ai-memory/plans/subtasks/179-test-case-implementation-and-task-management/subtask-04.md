# Subtask 04: Steps 8 & 9 - Auto-Switcher Epoch Drift Compensator & Split-SQLite Index Fragmentation Scorer

- **Parent Plan**: `.ai-memory/plans/pending/179-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/179-test-case-implementation-and-task-management/spec.md#step-8-auto-switcher-quota-reset-epoch-drift-compensator-imp-551`
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/epoch_drift_compensator.rs`
  - `src-tauri/src/modules/auto_switcher/tests_epoch_drift_compensator.rs`
  - `src-tauri/src/modules/db_orm/index_fragmentation_scorer.rs`
  - `src-tauri/src/modules/db_orm/tests_index_fragmentation_scorer.rs`
- **Status**: [COMPLETED]

## Description
1. Step 8 (Imp-551): Implemented `src-tauri/src/modules/auto_switcher/epoch_drift_compensator.rs` providing 30s safety buffering, smoothed drift estimation, and outlier clamping (300s). Authored unit test suite `tests_epoch_drift_compensator.rs` with 6 test scenarios. Registered in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. Step 9 (Imp-552): Implemented `src-tauri/src/modules/db_orm/index_fragmentation_scorer.rs` evaluating page density and out-of-order sequence penalties to produce `IndexMaintenanceRecommendation` (`None`, `AnalyzeOnly`, `ReindexRequired`). Authored unit test suite `tests_index_fragmentation_scorer.rs` with 5 test scenarios. Registered in `src-tauri/src/modules/db_orm/mod.rs`.

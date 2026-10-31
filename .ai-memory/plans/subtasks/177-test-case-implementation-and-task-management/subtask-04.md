# Subtask 04: Steps 8 & 9 - Auto-Switcher Quota Recovery Forecaster & Split-SQLite WAL Fragmentation Surveyor

- **Parent Plan**: `.ai-memory/plans/pending/177-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/177-test-case-implementation-and-task-management/spec.md#step-8-auto-switcher-candidate-quota-recovery-forecaster-imp-543`
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/quota_recovery_forecaster.rs`
  - `src-tauri/src/modules/auto_switcher/tests_quota_recovery_forecaster.rs`
  - `src-tauri/src/modules/db_orm/wal_fragmentation_surveyor.rs`
  - `src-tauri/src/modules/db_orm/tests_wal_fragmentation_surveyor.rs`
- **Status**: [COMPLETED]

## Description
1. Step 8 (Imp-543): Implemented `src-tauri/src/modules/auto_switcher/quota_recovery_forecaster.rs` providing `QuotaRecoveryForecaster` with rolling window estimation, replenishment horizon bounds, and saturation clamps. Authored unit test suite `tests_quota_recovery_forecaster.rs` with 6 test scenarios. Registered in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. Step 9 (Imp-544): Implemented `src-tauri/src/modules/db_orm/wal_fragmentation_surveyor.rs` inspecting WAL size, uncheckpointed page count, and fragmentation percentages to yield `CheckpointRecommendation::Passive` or `CheckpointRecommendation::Truncate`. Authored unit test suite `tests_wal_fragmentation_surveyor.rs` with 6 test scenarios. Registered in `src-tauri/src/modules/db_orm/mod.rs`.

# Subtask 05: Transient Test Sandbox Lifecycle Guard (`Imp-694`)
Sequence: 213
Step: 280
Status: COMPLETED
Module: `src-tauri/src/modules/release_gate/transient_test_sandbox_lifecycle_guard.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs`

## Description
Enforces AGENTS.md rule: "Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants." Manages sandbox lifecycle states, registers created test sandboxes, audits cleanup completion, verifies termination of orphaned child processes, and purges sandbox temporary directories.

## Implemented Components
1. `TRANSIENT_SANDBOX_PREFIX = "agm-test-sandbox-"`: Standardized transient test prefix.
2. `SandboxStatus` enum: `Created`, `Active`, `Cleaned`, `LeakedRemnants` with state query helpers (`is_active`, `is_cleaned`, `is_leaked`) and Display implementation.
3. `TestSandboxRecord` struct: Tracks `sandbox_id`, `base_dir`, `created_epoch_ms`, `spawned_pids`, and `status`.
4. `SandboxCleanupReport` struct: Reports `sandbox_id`, `is_clean`, `killed_processes_count`, `deleted_files_count`, `surviving_pids`, `surviving_paths`, `violations`, with summary formatting and Display implementation.
5. Lifecycle methods and helpers:
   - `register_sandbox`: Creates sandbox record in `Created` status.
   - `track_child_process`: Records spawned PID and transitions to `Active` status.
   - `audit_sandbox_cleanliness`: Audits active OS PIDs against spawned sandbox PIDs and filesystem remnants against base directory; computes surviving remnants, killed count, and violations.
   - `audit_and_update_sandbox`: Audits cleanliness and updates record state to `Cleaned` or `LeakedRemnants`.
6. Registered in `src-tauri/src/modules/release_gate/mod.rs` with `pub mod`, `#[cfg(test)] pub mod`, and `pub use`.
7. Comprehensive unit test suite in `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs`.

# Subtask 02: Steps 3, 4 & 5 - Multi-Instance Stale PID Reaper, Profile Hasher & Binary Probe Tests

- **Parent Plan**: `.ai-memory/plans/completed/176-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/176-test-case-implementation-and-task-management/spec.md#step-3-stale-instance-pid-process-reaper-unit-test-suite`
- **Target Files**:
  - `src-tauri/src/modules/instance/tests_stale_pid_reaper.rs`
  - `src-tauri/src/modules/instance/tests_profile_hasher.rs`
  - `src-tauri/src/modules/instance/tests_binary_execution_probe.rs`
- **Status**: [COMPLETED]

## Execution Summary
1. Step 3: Implemented unit test suite in `tests_stale_pid_reaper.rs` validating alive PID retention, dead PID reaping, and zero PID immunity. Registered in `instance.rs`.
2. Step 4: Implemented unit test suite in `tests_profile_hasher.rs` validating deterministic SHA-256 state hashing, field-by-field drift detection, and canonical byte streams. Registered in `instance.rs`.
3. Step 5: Implemented unit test suite in `tests_binary_execution_probe.rs` validating missing binary rejection, directory rejection, 0-byte file rejection, host executable validation, and serde JSON contract consistency. Registered in `instance.rs`.

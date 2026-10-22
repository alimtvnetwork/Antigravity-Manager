# Subtask 02: Steps 3, 4 & 5 - Multi-Instance Crash Loop Backoff, Crash Loop Guard & Health Monitor Tests

- **Parent Plan**: `.ai-memory/plans/pending/179-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/179-test-case-implementation-and-task-management/spec.md#step-3-multi-instance-crash-loop-backoff-guard-unit-test-suite`
- **Target Files**:
  - `src-tauri/src/modules/instance/tests_crash_loop_backoff.rs`
  - `src-tauri/src/modules/instance/tests_crash_loop_guard.rs`
  - `src-tauri/src/modules/instance/tests_child_health_monitor.rs`
- **Status**: [COMPLETED]

## Description
1. Step 3: Authored `src-tauri/src/modules/instance/tests_crash_loop_backoff.rs` with 5 unit test scenarios verifying exponential backoff (2s..16s), short runtime detection (<10s), healthy resets, and 30s ceiling clamps. Registered in `src-tauri/src/modules/instance.rs`.
2. Step 4: Authored `src-tauri/src/modules/instance/tests_crash_loop_guard.rs` with 5 unit test scenarios validating rapid crash quarantine triggering (3 rapid crashes < 5s), quarantine isolation, and manual reset. Registered in `src-tauri/src/modules/instance.rs`.
3. Step 5: Authored `src-tauri/src/modules/instance/tests_child_health_monitor.rs` with 5 unit test scenarios validating pulse recording, silence duration tracking, stall threshold detection (>45s), and instance removal. Registered in `src-tauri/src/modules/instance.rs`.

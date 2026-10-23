# Subtask 05: Transient Test Sandbox Lifecycle Guard (`Imp-694`)
Sequence: 213
Step: 280
Status: IN_PROGRESS
Module: `src-tauri/src/modules/release_gate/transient_test_sandbox_lifecycle_guard.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs`

## Description
Enforces AGENTS.md rule: "Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants." Manages sandbox lifecycle states, registers created test sandboxes, audits cleanup completion, verifies termination of orphaned child processes, and purges sandbox temporary directories.

# Subtask 04: Transient Sandbox Lifecycle & Cleanup Sentinel (`Imp-713`)
Sequence: 217
Step: 299
Status: COMPLETED
Module: `src-tauri/src/modules/instance/transient_sandbox_cleanup_sentinel.rs`

## Description
Implement the Transient Sandbox Lifecycle & Cleanup Sentinel adhering to specification `02-spec/21-app/217-implementing-next-five-tasks/spec.md`:
1. Enforce AGENTS.md Caller IDE Self-Preservation & Transient Test Sandboxing:
   "Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided. Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants."
2. Implement `TransientSandboxState`, `SandboxInstanceDescriptor`, `SandboxEvictionVerdict`, and `SandboxCleanupReport`.
3. Provide `evaluate_sandbox_eviction_safety` and `plan_transient_sandbox_cleanup`.
4. Create companion unit tests in `src-tauri/src/modules/instance/tests_transient_sandbox_cleanup_sentinel.rs`.

## Verification & Outcomes
- Implemented `TransientSandboxState` enum (`Created`, `Active`, `PendingCleanup`, `Purged`, `OrphanRemnantsDetected`).
- Implemented `SandboxInstanceDescriptor`, `SandboxEvictionVerdict`, and `SandboxCleanupReport`.
- Implemented `evaluate_sandbox_eviction_safety` enforcing exit code 2 fail-fast when caller IDE is targeted without `--force`.
- Implemented `plan_transient_sandbox_cleanup` planning complete eradication of sandbox directories and processes.
- Unit tests written in `tests_transient_sandbox_cleanup_sentinel.rs` verifying caller self-preservation, exit code 2 fail-fast, and zero-orphan cleanup.

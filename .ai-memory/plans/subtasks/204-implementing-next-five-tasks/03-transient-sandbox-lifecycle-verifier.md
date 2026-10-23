# Subtask 03: Transient Sandbox Lifecycle & Zero-Remnants Verifier (`Imp-647`)
Status: IN_PROGRESS
Parent Plan: 204-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Caller IDE Self-Preservation & Transient Test Sandboxing:
"Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants."

Implement `src-tauri/src/modules/instance/transient_sandbox_lifecycle_verifier.rs` and unit tests in `src-tauri/src/modules/instance/tests_transient_sandbox_lifecycle_verifier.rs`.

## Requirements
- Verify sandbox creation: unique sandbox ID, isolated temp storage path under `target/sandboxes/<id>`, isolated ports.
- Caller preservation: verify that caller process ID is detected and never terminated unless `--force` is provided.
- Verify sandbox cleanup: verify that all tracked child processes are terminated, sandboxed profile directory is fully removed, and zero orphan processes or filesystem remnants remain.
- Return structured `SandboxLifecycleReport` with verification status, remnants count, and cleanup violations.

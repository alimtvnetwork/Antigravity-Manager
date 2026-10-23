# Sequence 198 Specification: Implementing Next Five Tasks (Steps 206–210)

## Overview
This specification details the design, constraints, and verification requirements for Batch 198 (Steps 206–210 of 255) in Antigravity-Manager.

### Tasks Summary
1. **Task 1 (`Imp-620`)**: `src-tauri/src/proxy/upstream/transport_backoff_stepper.rs` & `tests_transport_backoff_stepper.rs`
   - Adaptive exponential backoff with full jitter and decorrelated jitter on upstream proxy network errors (DNS resolution failures, TCP resets, timeouts, 502/503/504 gateway failures). Incorporates retry quotas and circuit-breaker backoff step ladder per AGENTS.md retry requirements ("Bound every wait with a timeout", "exponential backoff with jitter").
2. **Task 2 (`Imp-621`)**: `src-tauri/src/modules/instance/transient_sandbox_manager.rs` & `tests_transient_sandbox_manager.rs`
   - Lifecycle manager for transient test sandboxes (`agm instances test-create` / `test-clean`). Generates isolated profile directories, mock settings, tracks active sandbox PID / GUID allocations, and executes complete non-orphan cleanup leaving zero filesystem or process remnants, while strictly preserving `workspaceStorage`.
3. **Task 3 (`Imp-622`)**: `src-tauri/src/modules/db_orm/fifo_prompt_queue_coordinator.rs` & `tests_fifo_prompt_queue_coordinator.rs`
   - Runtime prompt restore, dispatch, and queue processing coordinator across split SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`). Enforces strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`, disallowing LIFO queries in dispatch logic, and managing non-blocking lease locks with lease expiration timeouts.
4. **Task 4 (`Imp-623`)**: `src-tauri/src/modules/auto_switcher/account_switch_audit_emitter.rs` & `tests_account_switch_audit_emitter.rs`
   - Structured JSON telemetry and audit event emission for auto-switch decisions. Tracks switch trigger reasons (`RateLimit429`, `PreemptiveExhaustionVelocity`, `ManualOverride`, `HealthDecay`), candidate scores breakdown, cooldown duration, velocity forecast, and emitting auditable switch logs into `AppError`/tracing streams without swallowing errors.
5. **Task 5 (`Imp-624`)**: `src-tauri/src/modules/release_gate/rejection_breakdown_generator.rs` & `tests_rejection_breakdown_generator.rs`
   - Enforce AGENTS.md Contributor Respect & Attribution guideline: "Accompany every rejection with a concrete file-by-file accept/drop breakdown and an actionable path forward." Parses git diffs / pull request changesets, categorizes files into Accept / Drop / Partial, provides specific rationale per file, and generates markdown rejection reports.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All queries must use `ORDER BY created_at ASC, id ASC`.

# End-to-End Master Test Plan: Sequence 213 (Steps 276–280)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, proxy, UI, database, caller protection, and release governance modules implemented in Sequence 213 (`Imp-690` through `Imp-694`).
In accordance with the **Zero-Build & Zero-Test Invariant**, live test execution is deferred until an explicit testing prompt is issued.
Any AI agent executing deferred end-to-end verification must follow the detailed scenarios cataloged below.

---

## 2. Architectural Invariants Enforced
All modules strictly adhere to the non-negotiable architectural invariants codified in `AGENTS.md`:
- **Architecture & Pipeline First (Streaming Event Frame Parser)**: Parses and validates raw SSE / HTTP event streams into discrete Gemini protocol chunks, verifying boundary frames, detecting unclosed data payloads, parsing multi-line `data: ` blocks according to W3C SSE standards, and guaranteeing zero event loss or buffer truncation during high-throughput proxy streaming.
- **Minimalist & Contextual UI (Inline Select Dropdown Styler)**: Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects.
- **FIFO Restore & Queue Invariant (FIFO Queue Compaction Guard)**: All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Compaction, historical archiving, and tombstone purging passes must never invert row sequences or create sequence gaps.
- **Caller IDE Self-Preservation (Caller IDE Exit Code 2 Enforcer)**: Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided, preventing automated test daemons or CLI scripts from terminating the developer's working IDE session.
- **Transient Test Sandboxing & Zero Remnants (Transient Test Sandbox Lifecycle Guard)**: Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants.

---

## 3. Step-by-Step Task Mapping
| Step | Improvement ID | Category / Subsystem | Target File | Verification Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| **Step 276** | `Imp-690` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/streaming_event_frame_parser.rs` | `src-tauri/src/proxy/handlers/common/tests_streaming_event_frame_parser.rs` |
| **Step 277** | `Imp-691` | `modules::instance` | `src-tauri/src/modules/instance/inline_select_dropdown_styler.rs` | `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs` |
| **Step 278** | `Imp-692` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_queue_compaction_guard.rs` | `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs` |
| **Step 279** | `Imp-693` | `modules::instance` | `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs` | `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs` |
| **Step 280** | `Imp-694` | `modules::release_gate` | `src-tauri/src/modules/release_gate/transient_test_sandbox_lifecycle_guard.rs` | `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs` |

---

## 4. Comprehensive End-to-End Test Scenarios

### Test Case `E2E-213-01`: Streaming Event Frame Parser
- **Target Module**: `src-tauri/src/proxy/handlers/common/streaming_event_frame_parser.rs`
- **Unit Suite**: `src-tauri/src/proxy/handlers/common/tests_streaming_event_frame_parser.rs`
- **Improvement ID**: `Imp-690` (Step 276)
- **Invariant Addressed**: Architecture & Pipeline First (`AGENTS.md`)
- **Preconditions**:
  - Raw chunked HTTP response streams from upstream AI providers arriving over streaming connections.
- **Actions / Execution Steps**:
  1. Stream incoming raw SSE chunks with standard `data: ...` frames, verifying single and multiple events per chunk.
  2. Parse multi-line `data: ` blocks concatenated with newlines according to W3C SSE standard specifications.
  3. Validate partial event boundaries split across multiple packet chunks (e.g. data split halfway across bytes), ensuring proper buffer accumulation and zero event loss.
  4. Verify comment lines (`: heartbeat`), custom event names (`event: ping`), and empty keepalive lines are cleanly filtered.
  5. Validate terminal stream frames (`data: [DONE]`) trigger clean EOF state without JSON parsing errors.
- **Expected Outcomes**:
  - Full W3C SSE compliance with multi-line `data:` concatenation.
  - Zero dropped chunks, truncated events, or spurious parsing panics during high-throughput proxy streaming.

---

### Test Case `E2E-213-02`: Inline Select Dropdown Styler
- **Target Module**: `src-tauri/src/modules/instance/inline_select_dropdown_styler.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs`
- **Improvement ID**: `Imp-691` (Step 277)
- **Invariant Addressed**: Minimalist & Contextual UI (Segmented Capsule Invariant) (`AGENTS.md`)
- **Preconditions**:
  - UI markup or components containing inline `<select>` dropdowns intended for embedding within segmented capsules.
- **Actions / Execution Steps**:
  1. Submit `<select>` dropdown markup or class specifications mounted inside segmented capsules.
  2. Detect and flag prohibited opaque styles: default white background (`bg-white`), explicit borders (`border`, `border-gray-200`), and focus rings (`focus:ring-2`).
  3. Verify enforcement of mandatory canonical Tailwind styling classes: `bg-transparent border-0 focus:ring-0 focus:outline-none`.
  4. Test automated styling sanitizer / class injector: transforms defective dropdown class lists into fully compliant borderless transparent capsules.
  5. Distinguish standalone non-capsule dropdowns from inline segmented capsule dropdowns, applying strict validation only where nested pill-in-pill visual defects would occur.
- **Expected Outcomes**:
  - Complete elimination of nested white pill-in-pill visual defects.
  - Consistent dark-glass and transparent dropdown styling across all window control and toolbar capsules.

---

### Test Case `E2E-213-03`: FIFO Queue Compaction Guard
- **Target Module**: `src-tauri/src/modules/db_orm/fifo_queue_compaction_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs`
- **Improvement ID**: `Imp-692` (Step 278)
- **Invariant Addressed**: FIFO Restore & Queue Invariant (`AGENTS.md`)
- **Preconditions**:
  - Prompt queue tables (`active_prompts`, `prompt_backups`, `repo_prompts.db`) undergoing queue compaction, historical archiving, or tombstone cleanup.
- **Actions / Execution Steps**:
  1. Submit compaction query SQL to `validate_compaction_query_sql`:
     - Assert queries enforcing `ORDER BY created_at ASC, id ASC` pass (`Ok(())`).
     - Assert inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`, `ORDER BY id DESC`) fail with `InvertedLifoOrder`.
     - Assert queries missing secondary tie-breaker `id ASC` fail with `MissingIdTieBreaker`.
     - Assert queries lacking `ORDER BY` fail with `MissingOrderByClause`.
  2. Execute batch compaction via `execute_fifo_compaction`:
     - Feed sequence of `QueueRow` records with monotonically increasing `(created_at, id)` and interleaved tombstones.
     - Assert all tombstones (`is_tombstone == true` or `status == "tombstone"|"purged"|"deleted"`) are purged and counted in `tombstones_purged`.
     - Assert retained rows strictly preserve original chronological order with `is_order_preserved == true`.
     - Feed out-of-order sequence with timestamp regression: assert flagged with `TimestampRegression` and `is_order_preserved == false`.
     - Feed sequence with identical timestamps but inverted IDs: assert flagged with `IdTieBreakerInversion`.
     - Feed duplicate row IDs: assert flagged with `DuplicateId`.
  3. Verify canonical SQL builders `build_canonical_compaction_query` and `build_canonical_tombstone_purge_query`.
  4. Verify `enforce_compaction_order_by` replaces non-compliant clauses while preserving trailing `LIMIT`/`OFFSET`.
- **Expected Outcomes**:
  - Strict preservation of original FIFO order during compaction.
  - Safe purge of tombstoned records without sequence regressions or duplicate row processing.

---

### Test Case `E2E-213-04`: Caller IDE Exit Code 2 Enforcer
- **Target Module**: `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs`
- **Improvement ID**: `Imp-693` (Step 279)
- **Invariant Addressed**: Caller IDE Self-Preservation & Exit Code 2 Invariant (`AGENTS.md`)
- **Preconditions**:
  - Active caller IDE session identified by instance ID (e.g. `inst-caller-active`) or environment variable.
- **Actions / Execution Steps**:
  1. Evaluate instance commands targeting the caller instance (`agm instances stop`, `rm`, `switch`):
     - Without `--force`: assert evaluation returns `EnforcementDecision::BlockedCallerSelfTermination`, formatting exit code 2 and diagnostic warning.
     - With `--force` (`force: true`): assert evaluation returns `EnforcementDecision::AllowedWithOverride`.
  2. Evaluate non-destructive commands targeting caller instance (`status`, `info`, `logs`):
     - Assert permitted to execute normally without blocking.
  3. Evaluate destructive commands targeting other, non-caller instances (`inst-other-worker`):
     - Assert permitted to proceed without requiring `--force`.
  4. Verify formatted terminal error diagnostic message clearly advises the user on why the command failed and how to bypass with `--force`.
- **Expected Outcomes**:
  - Guaranteed prevention of accidental caller IDE termination.
  - Strict exit code 2 compliance on intercepted commands.

---

### Test Case `E2E-213-05`: Transient Test Sandbox Lifecycle Guard
- **Target Module**: `src-tauri/src/modules/release_gate/transient_test_sandbox_lifecycle_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs`
- **Improvement ID**: `Imp-694` (Step 280)
- **Invariant Addressed**: Caller IDE Self-Preservation & Transient Test Sandboxing (`AGENTS.md`)
- **Preconditions**:
  - Integration test runs or automated test suites executing `agm instances test-create`.
- **Actions / Execution Steps**:
  1. Register transient test sandboxes with unique ephemeral instance IDs, isolated scratch paths, and registered child PIDs.
  2. Audit active sandboxes to track registered resources, open file locks, and running worker processes.
  3. Execute cleanup teardown via `agm instances test-clean`:
     - Assert all registered subprocesses receive termination signals and are confirmed dead.
     - Assert sandbox filesystem trees are deleted completely.
     - Assert zero leftover directory remnants, locks, or orphan processes.
  4. Simulate abnormal test termination (process crash before clean call):
     - Trigger sweep phase; assert orphaned test sandboxes and associated processes are identified and swept.
- **Expected Outcomes**:
  - Complete lifecycle tracking and deterministic cleanup of all transient test sandboxes.
  - 100% clean filesystem and zero leftover orphan processes.

---

## 5. Exact Targeted Deferred Test Commands
When an explicit testing prompt is issued, run the following commands sequentially:

```bash
# Step 276 (Imp-690): Streaming Event Frame Parser
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_streaming_event_frame_parser

# Step 277 (Imp-691): Inline Select Dropdown Styler
cargo test --package antigravity-manager --lib modules::instance::tests_inline_select_dropdown_styler

# Step 278 (Imp-692): FIFO Queue Compaction Guard
cargo test --package antigravity-manager --lib modules::db_orm::tests_fifo_queue_compaction_guard

# Step 279 (Imp-693): Caller IDE Exit Code 2 Enforcer
cargo test --package antigravity-manager --lib modules::instance::tests_caller_ide_exit_code_enforcer

# Step 280 (Imp-694): Transient Test Sandbox Lifecycle Guard
cargo test --package antigravity-manager --lib modules::release_gate::tests_transient_test_sandbox_lifecycle_guard
```

---

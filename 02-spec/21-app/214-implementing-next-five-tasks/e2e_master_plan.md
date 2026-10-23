# End-to-End Master Test Plan: Sequence 214 (Steps 281–285)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, proxy streaming, UI capsule, database pagination, process execution, and contributor governance modules implemented in Sequence 214 (`Imp-695` through `Imp-699`).
In accordance with the **Zero-Build & Zero-Test Invariant**, live test execution is deferred until an explicit testing prompt is issued.
Any AI agent executing deferred end-to-end verification must follow the detailed scenarios cataloged below.

---

## 2. Architectural Invariants Enforced
All modules strictly adhere to the non-negotiable architectural invariants codified in `AGENTS.md`:
- **Architecture & Pipeline First (Streaming Payload Accumulator)**: Accumulates and validates streaming response chunks into cohesive Gemini candidate turns, tracking token boundaries, thinking block delimiters (`<thought>...</thought>`), and tool invocation fragments, bounding maximum stream buffer size to prevent memory bloat while ensuring complete turn delivery.
- **Minimalist & Contextual UI (Capsule Segmented Button Group Styler)**: When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons.
- **FIFO Restore & Queue Invariant (FIFO Prompt Restore Cursor Stepper)**: Implements cursor-driven FIFO query stepping (`WHERE (created_at > :cursor_time) OR (created_at = :cursor_time AND id > :cursor_id) ORDER BY created_at ASC, id ASC LIMIT :batch_size`) across prompt queues without sequence inversions.
- **Process Spawning & Job Breakaway Invariant (Batch File Execution Interceptor)**: Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`).
- **Contributor Respect & Attribution (Contributor Authorship Preservation Guard)**: Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs. Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements.

---

## 3. Step-by-Step Task Mapping
| Step | Improvement ID | Category / Subsystem | Target File | Verification Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| **Step 281** | `Imp-695` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/streaming_payload_accumulator.rs` | `src-tauri/src/proxy/handlers/common/tests_streaming_payload_accumulator.rs` |
| **Step 282** | `Imp-696` | `modules::instance` | `src-tauri/src/modules/instance/capsule_segmented_button_group_styler.rs` | `src-tauri/src/modules/instance/tests_capsule_segmented_button_group_styler.rs` |
| **Step 283** | `Imp-697` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_prompt_restore_cursor_stepper.rs` | `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_cursor_stepper.rs` |
| **Step 284** | `Imp-698` | `modules::instance` | `src-tauri/src/modules/instance/batch_file_execution_interceptor.rs` | `src-tauri/src/modules/instance/tests_batch_file_execution_interceptor.rs` |
| **Step 285** | `Imp-699` | `modules::release_gate` | `src-tauri/src/modules/release_gate/contributor_authorship_preservation_guard.rs` | `src-tauri/src/modules/release_gate/tests_contributor_authorship_preservation_guard.rs` |

---

## 4. Comprehensive End-to-End Test Scenarios

### Test Case `E2E-214-01`: Streaming Payload Accumulator
- **Target Module**: `src-tauri/src/proxy/handlers/common/streaming_payload_accumulator.rs`
- **Unit Suite**: `src-tauri/src/proxy/handlers/common/tests_streaming_payload_accumulator.rs`
- **Improvement ID**: `Imp-695` (Step 281)
- **Invariant Addressed**: Architecture & Pipeline First (`AGENTS.md`)
- **Actions / Execution Steps**:
  1. Stream multiple incremental text parts and verify accumulation into unified content parts.
  2. Detect `<thought>...</thought>` delimiters and partition into dedicated thinking blocks (`thought: true`).
  3. Detect fragmented function call JSON chunks and aggregate until complete valid JSON is reached.
  4. Enforce buffer memory ceiling (`max_buffer_bytes`), returning clear overflow diagnostics without crashing.

---

### Test Case `E2E-214-02`: Capsule Segmented Button Group Styler
- **Target Module**: `src-tauri/src/modules/instance/capsule_segmented_button_group_styler.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_capsule_segmented_button_group_styler.rs`
- **Improvement ID**: `Imp-696` (Step 282)
- **Invariant Addressed**: Minimalist & Contextual UI (Contiguous Pill Capsule Invariant) (`AGENTS.md`)
- **Actions / Execution Steps**:
  1. Audit markup for adjacent button clusters (window controls, action toolbars).
  2. Detect prohibited loose circular buttons (`rounded-full p-2` without shared container) and missing dividers.
  3. Validate canonical segmented capsule wrapping (`inline-flex items-center rounded-full border border-zinc-800/80 bg-zinc-950/60 backdrop-blur-md`).
  4. Generate and normalize button group CSS/HTML markup.

---

### Test Case `E2E-214-03`: FIFO Prompt Restore Cursor Stepper
- **Target Module**: `src-tauri/src/modules/db_orm/fifo_prompt_restore_cursor_stepper.rs`
- **Unit Suite**: `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_cursor_stepper.rs`
- **Improvement ID**: `Imp-697` (Step 283)
- **Invariant Addressed**: FIFO Restore & Queue Invariant (`AGENTS.md`)
- **Actions / Execution Steps**:
  1. Generate cursor SQL queries with initial cursor (None) and intermediate cursors `(cursor_time, cursor_id)`.
  2. Assert generated SQL strictly enforces `ORDER BY created_at ASC, id ASC`.
  3. Validate cursor stepping logic across paginated test datasets, asserting zero duplicate items, zero dropped items, and strict monotonic ordering.
  4. Validate prevention of inverted LIFO stepping clauses.

---

### Test Case `E2E-214-04`: Batch File Execution Interceptor
- **Target Module**: `src-tauri/src/modules/instance/batch_file_execution_interceptor.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_batch_file_execution_interceptor.rs`
- **Improvement ID**: `Imp-698` (Step 284)
- **Invariant Addressed**: Process Spawning & Job Breakaway Invariant (`AGENTS.md`)
- **Actions / Execution Steps**:
  1. Test executable paths ending in `.cmd` or `.bat` (case-insensitive).
  2. Assert `is_batch_file` returns true.
  3. Intercept direct command execution and transform into Windows Shell dispatch: `cmd.exe /c call "<script>" <args>`.
  4. Ensure standard executables (`.exe`, `/bin/sh`) are preserved without unnecessary wrapping.

---

### Test Case `E2E-214-05`: Contributor Authorship Preservation Guard
- **Target Module**: `src-tauri/src/modules/release_gate/contributor_authorship_preservation_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_contributor_authorship_preservation_guard.rs`
- **Improvement ID**: `Imp-699` (Step 285)
- **Invariant Addressed**: Contributor Respect & Attribution (`AGENTS.md`)
- **Actions / Execution Steps**:
  1. Validate commit messages and PR bodies for multi-author or proxy contributions.
  2. Assert explicit `Co-authored-by: Name <email>` trailers are required and validated.
  3. Verify disclosure of affected existing behaviors and unverified paths.
  4. Verify rejection messages provide concrete file-by-file accept/drop breakdowns and actionable guidance.

---

## 5. Exact Targeted Deferred Test Commands
When an explicit testing prompt is issued, run the following commands sequentially:

```bash
# Step 281 (Imp-695): Streaming Payload Accumulator
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_streaming_payload_accumulator

# Step 282 (Imp-696): Capsule Segmented Button Group Styler
cargo test --package antigravity-manager --lib modules::instance::tests_capsule_segmented_button_group_styler

# Step 283 (Imp-697): FIFO Prompt Restore Cursor Stepper
cargo test --package antigravity-manager --lib modules::db_orm::tests_fifo_prompt_restore_cursor_stepper

# Step 284 (Imp-698): Batch File Execution Interceptor
cargo test --package antigravity-manager --lib modules::instance::tests_batch_file_execution_interceptor

# Step 285 (Imp-699): Contributor Authorship Preservation Guard
cargo test --package antigravity-manager --lib modules::release_gate::tests_contributor_authorship_preservation_guard
```

---

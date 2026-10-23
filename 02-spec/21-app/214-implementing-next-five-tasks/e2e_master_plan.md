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
- **FIFO Restore & Queue Invariant (FIFO Prompt Restore Cursor Stepper)**: Implements stateful cursor-driven FIFO query stepping (`WHERE (created_at > ?) OR (created_at = ? AND id > ?) ORDER BY created_at ASC, id ASC LIMIT ?`) across SQLite prompt queues (`active_prompts`, `prompt_backups`, `repo_prompts.db`) to guarantee deterministic pagination without unbounded memory usage or sequence inversions.
- **Process Spawning & Job Breakaway Invariant (Batch File Execution Interceptor)**: Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`).
- **Contributor Respect & Attribution (Contributor Authorship Preservation Guard)**: Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs. Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements. Accompany every rejection with a concrete file-by-file accept/drop breakdown and an actionable path forward.

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
- **Preconditions**:
  - Live HTTP streaming responses from Gemini, Claude, or OpenAI adapter protocols emitting partial text delta chunks and fragmented JSON tokens.
- **Actions / Execution Steps**:
  1. **Incremental Text Chunk Accumulation**: Stream partial text tokens (e.g. `["Hello", " ", "world", "!"]`) and assert that the accumulator merges them into a unified candidate content turn with complete character preservation.
  2. **Thinking Block Partitioning**:
     - Stream thinking blocks delimited by `<thought>...</thought>` or `<thinking>...</thinking>` tags across chunk boundaries.
     - Verify that the accumulator extracts thinking content into dedicated thinking segments (`thought: true`), stripping delimiters from public assistant visible text.
  3. **Fragmented Tool Call JSON Reassembly**:
     - Stream incomplete JSON fragments for function call arguments across multiple frames (e.g. `{"query": `, `"search `, `term"}`).
     - Assert that the accumulator buffers argument fragments until valid JSON is formed before emitting the complete tool invocation structure.
  4. **Buffer Memory Ceiling Enforcement**:
     - Configure `max_buffer_bytes` limit (e.g. 1 MB or 4 MB).
     - Stream an oversized, malicious, or unclosed payload exceeding the configured buffer ceiling.
     - Assert that the accumulator returns an explicit overflow error (`PayloadBufferOverflow`) with byte count diagnostics without panicking or leaking memory.
  5. **Stream Termination & Candidate State Verification**:
     - Feed stream completion indicators (`finish_reason: "STOP"`, `[DONE]`).
     - Verify that intermediate buffers are flushed, token counts are finalized, and final candidate turns match Gemini protocol structural standards.
- **Expected Outcomes**:
  - Full W3C and Gemini streaming compliance.
  - Complete isolation of thinking tokens from visible content.
  - Zero memory leaks or unbounded buffer accumulation during long-lived proxy streams.

---

### Test Case `E2E-214-02`: Capsule Segmented Button Group Styler
- **Target Module**: `src-tauri/src/modules/instance/capsule_segmented_button_group_styler.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_capsule_segmented_button_group_styler.rs`
- **Improvement ID**: `Imp-696` (Step 282)
- **Invariant Addressed**: Minimalist & Contextual UI (Contiguous Pill Capsule Invariant) (`AGENTS.md`)
- **Preconditions**:
  - Desktop window control clusters (minimize, maximize, close), instance action toolbars, or adjacent multi-button controls rendered in UI views.
- **Actions / Execution Steps**:
  1. **Audit Button Cluster Markup**:
     - Inspect markup or styling classes for adjacent button clusters.
     - Detect prohibited loose circular button patterns (`rounded-full p-2` on sibling buttons without a shared segmented container).
  2. **Enforce Canonical Capsule Wrapper**:
     - Verify that adjacent button groups are wrapped in a single contiguous segmented pill container:
       `inline-flex items-center rounded-full border border-zinc-800/80 bg-zinc-950/60 backdrop-blur-md`.
  3. **Validate Subtle Divider Separators**:
     - Assert that internal dividers between adjacent buttons within the capsule use subtle divider borders (`divide-x divide-zinc-800/80` or individual subtle divider borders) rather than harsh lines or gaps.
  4. **Enforce Inline Dropdown Styling Inside Capsules**:
     - Verify that any inline `<select>` element mounted inside a segmented capsule uses `bg-transparent border-0 focus:ring-0 focus:outline-none` to eliminate nested white pill-in-pill visual defects.
  5. **Automated Markup Transformation / Sanitization**:
     - Feed uncompliant loose button cluster configurations to the styler.
     - Assert that the styler generates compliant contiguous segmented dark-glass capsule markup and styling classes.
- **Expected Outcomes**:
  - Total elimination of loose, disjointed circular buttons.
  - Uniform, cohesive dark-glass segmented pill styling across all window controls and toolbars.

---

### Test Case `E2E-214-03`: FIFO Prompt Restore Cursor Stepper
- **Target Module**: `src-tauri/src/modules/db_orm/fifo_prompt_restore_cursor_stepper.rs`
- **Unit Suite**: `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_cursor_stepper.rs`
- **Improvement ID**: `Imp-697` (Step 283)
- **Invariant Addressed**: FIFO Restore & Queue Invariant (`AGENTS.md`)
- **Preconditions**:
  - SQLite prompt queue tables (`active_prompts`, `prompt_backups`, `repo_prompts.db`) containing hundreds of prompt records requiring deterministic paginated drains.
- **Actions / Execution Steps**:
  1. **Initial Cursor Query Generation**:
     - Call `build_fifo_cursor_query(table, None, batch_size)`.
     - Assert SQL matches `SELECT id, created_at FROM {table} ORDER BY created_at ASC, id ASC LIMIT ?`.
     - Assert query parameters equal `[batch_size]`.
  2. **Stepped Cursor Query Generation**:
     - Call `build_fifo_cursor_query(table, Some(cursor), batch_size)` with `cursor = (last_created_at, last_id)`.
     - Assert SQL matches `SELECT id, created_at FROM {table} WHERE (created_at > ?) OR (created_at = ? AND id > ?) ORDER BY created_at ASC, id ASC LIMIT ?`.
     - Assert query parameters equal `[last_created_at, last_created_at, last_id, batch_size]`.
  3. **SQL Invariant Validation**:
     - Pass compliant queries: assert `validate_cursor_query_sql(sql)` returns `Ok(())`.
     - Pass queries missing `ORDER BY`: assert intercepted with `MissingOrderByClause`.
     - Pass inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`): assert intercepted with `InvertedLIFOOrderDetected`.
     - Pass queries missing secondary tie-breaker `id ASC`: assert intercepted with `MissingTieBreakerIdAsc`.
     - Pass queries with disallowed sort columns (`ORDER BY priority ASC, id ASC`): assert intercepted with `DisallowedSortColumn`.
  4. **Monotonicity & Sequence Anomaly Interception**:
     - Call `step_fifo_cursor` with empty row slice: assert `retrieved_count: 0`, `next_cursor: None`, `is_exhausted: true`.
     - Call with sequential rows matching batch size: assert cursor updates to last row, `is_exhausted: false`.
     - Call with rows containing timestamp regression (`curr_created_at < prev_created_at`): assert `TimestampRegression` detected.
     - Call with rows containing identical timestamps and inverted IDs (`curr_id <= prev_id`): assert `IdTieBreakerInversion` detected.
     - Call with duplicate primary key IDs: assert `DuplicateId` detected.
  5. **Stateful Drain Lifecycle Simulation**:
     - Initialize `FifoPromptRestoreCursorStepper` with `batch_size = 3`.
     - Feed multi-batch test dataset through `next_query()` and `record_step()`.
     - Assert cursor advances monotonically across all steps, total retrieved count matches dataset size, and final step reports `is_exhausted == true`.
- **Expected Outcomes**:
  - 100% deterministic FIFO queue pagination.
  - Zero dropped rows, zero duplicate rows, and strict monotonic order preservation.
  - Immediate interception of any inverted LIFO query or sequence regression.

---

### Test Case `E2E-214-04`: Batch File Execution Interceptor
- **Target Module**: `src-tauri/src/modules/instance/batch_file_execution_interceptor.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_batch_file_execution_interceptor.rs`
- **Improvement ID**: `Imp-698` (Step 284)
- **Invariant Addressed**: Process Spawning & Job Breakaway Invariant (`AGENTS.md`)
- **Preconditions**:
  - Multi-instance launchers, CLI tools, or external process managers invoking child executables on Windows and cross-platform environments.
- **Actions / Execution Steps**:
  1. **File Extension Inspection**:
     - Inspect executable paths with `.cmd` or `.bat` extensions (case-insensitively, e.g. `.bat`, `.BAT`, `.cmd`, `.CMD`).
     - Assert `is_batch_file` correctly identifies batch scripts.
  2. **Direct Command Interception**:
     - Attempt to pass a batch file directly as the program path in process spawn configuration.
     - Assert that the interceptor halts direct execution to prevent `os error 5: Access is denied` under restricted Windows Job Objects.
  3. **Multi-Stage Fallback Ladder**:
     - **Stage 1 (Compiled Binary Resolution)**: Check if a corresponding compiled binary (`.exe`) exists alongside or within the target package (e.g. `npm.exe` instead of `npm.cmd`). If present, rewrite execution path directly to the binary.
     - **Stage 2 (Detected Base Binary)**: Inspect PATH for base runtime binary (e.g. `node.exe` or `git.exe`).
     - **Stage 3 (Windows Shell Dispatch)**: Transform batch invocation into Windows Shell dispatch: `cmd.exe /c call "<batch_script>" <args>` or `cmd.exe /c start`.
  4. **Non-Batch Executable Pass-Through**:
     - Verify standard binaries (`.exe`, `/bin/sh`, `/usr/bin/python3`) pass through without shell wrapping or overhead.
  5. **Job Breakaway Ladder Verification**:
     - Verify `spawn_with_job_breakaway_fallback`: attempts `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`), and falls back safely to standard process creation if running within an unprivileged Windows Job Object.
- **Expected Outcomes**:
  - Total prevention of Windows `os error 5: Access is denied` during multi-instance launching.
  - Deterministic multi-stage fallback ladder for all `.bat` and `.cmd` scripts.

---

### Test Case `E2E-214-05`: Contributor Authorship Preservation Guard
- **Target Module**: `src-tauri/src/modules/release_gate/contributor_authorship_preservation_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_contributor_authorship_preservation_guard.rs`
- **Improvement ID**: `Imp-699` (Step 285)
- **Invariant Addressed**: Contributor Respect & Attribution (`AGENTS.md`)
- **Preconditions**:
  - Git commit history, pull request metadata, and release gate validation during squash merging, proxy PR authoring, and PR review cycles.
- **Actions / Execution Steps**:
  1. **Authorship Preservation & Co-Author Trailers**:
     - Inspect commit messages and proxy PR descriptions for multi-contributor changes.
     - Assert mandatory presence and format of `Co-authored-by: Full Name <contributor@example.com>` trailers.
     - Verify contributor's original Git authorship is preserved during squash and merge operations.
  2. **Mandatory Cost & Impact Disclosure**:
     - Inspect PR description and release notes for required disclosures:
       - Assert mandatory presence of "Affected Existing Behaviors" section.
       - Assert mandatory presence of "Unverified Paths" section alongside improvements.
     - Block merging of any PR that claims improvements without disclosing known costs and unverified paths.
  3. **Constructive File-by-File Rejection Breakdown**:
     - Audit PR rejection payloads:
       - Verify every rejection includes a concrete, file-by-file accept/drop breakdown (`Accept: file A; Drop: file B`).
       - Verify rejection provides an actionable path forward without dismissing contributor effort.
  4. **Strict Attribution Invariant on Production Releases**:
     - Verify that production release changelogs (`changelog.md`, `changelog_en.md`) adhere to the release attribution policy (attributing strictly `@aukgit` in official release notes while preserving git history co-authorship trailers).
- **Expected Outcomes**:
  - 100% preservation of contributor authorship in Git commit metadata.
  - Transparent disclosure of costs and unverified paths prior to code integration.
  - Constructive, actionable review feedback on rejected PRs.

---

## 5. Verification Matrix & Edge Cases Summary

| Test Case | Subsystem | Happy Path Scenario | Primary Failure / Edge Case Mode | Invariant / Rule Checked |
| :--- | :--- | :--- | :--- | :--- |
| `E2E-214-01` | `proxy::handlers::common` | Cohesive turn accumulation across SSE chunks | Buffer overflow (> `max_buffer_bytes`), fragmented JSON | Architecture & Pipeline First |
| `E2E-214-02` | `modules::instance` | Seamless contiguous capsule segmented styling | Loose disjointed circular buttons, nested white pill dropdowns | Contiguous Pill Capsule Invariant |
| `E2E-214-03` | `modules::db_orm` | Deterministic monotonic FIFO cursor pagination | Inverted LIFO clauses, timestamp regressions, duplicate IDs | Strict FIFO Restore & Queue Invariant |
| `E2E-214-04` | `modules::instance` | Transparent batch file resolution to `.exe` or shell | Direct batch execution in Job Object (`os error 5`) | Process Spawning & Job Breakaway |
| `E2E-214-05` | `modules::release_gate` | Valid `Co-authored-by:` trailers & cost disclosures | Missing attribution, hidden costs, vague rejections | Contributor Respect & Attribution |

---

## 6. Exact Targeted Deferred Test Commands
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

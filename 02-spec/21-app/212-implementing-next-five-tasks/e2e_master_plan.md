# End-to-End Master Test Plan: Sequence 212 (Steps 271–275)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, proxy, UI, database, and release governance modules implemented in Sequence 212 (`Imp-685` through `Imp-689`).
In accordance with the **Zero-Build & Zero-Test Invariant**, live test execution is deferred until an explicit testing prompt is issued.
Any AI agent executing deferred end-to-end verification must follow the detailed scenarios cataloged below.

---

## 2. Architectural Invariants Enforced
All modules strictly adhere to the non-negotiable architectural invariants codified in `AGENTS.md`:
- **Pipeline First & Response Header Sanitization**: Strips hop-by-hop and internal upstream response headers (`server`, `alt-svc`, `x-goog-safety-ratings`, `x-goog-ratelimit-remaining`, `set-cookie`) and normalizes standard Antigravity streaming headers (`content-type: text/event-stream; charset=utf-8`, `cache-control: no-cache`, `connection: keep-alive`).
- **Minimalist & Contextual UI (Segmented Capsule Invariant)**: Adjacent desktop window controls (Minimize, Maximize, Close) MUST be formatted as contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling) to eliminate loose, disjointed circular buttons.
- **FIFO Restore & Queue Invariant**: Draining and spillover queries for dead-letter prompts across SQLite tables (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden, and original timestamps and lineage must be preserved.
- **Caller IDE Self-Preservation**: Traverses the operating system process ancestry tree (PPID chain) to detect caller IDE processes (`code.exe`, `cursor.exe`, `windsurf.exe`, `idea64.exe`), preventing automated switcher daemons or stop commands from terminating the developer's active working session.
- **Documentation & Attribution Parity Guard**: Enforces that GitHub release notes strictly attribute `@aukgit` (`(Thanks to @aukgit)`), reject all third-party handles (`@...`), forbid absolute filesystem paths (`D:\...`, `C:\...`, `/home/...`), and maintain exact tag parity with `CHANGELOG.md`.

---

## 3. Step-by-Step Task Mapping
| Step | Improvement ID | Module | Target File | Verification Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| Step 271 | `Imp-685` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/pipeline_response_header_sanitizer.rs` | `src-tauri/src/proxy/handlers/common/tests_pipeline_response_header_sanitizer.rs` |
| Step 272 | `Imp-686` | `modules::instance` | `src-tauri/src/modules/instance/window_control_capsule_formatter.rs` | `src-tauri/src/modules/instance/tests_window_control_capsule_formatter.rs` |
| Step 273 | `Imp-687` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_dead_letter_queue_spillover_guard.rs` | `src-tauri/src/modules/db_orm/tests_fifo_dead_letter_queue_spillover_guard.rs` |
| Step 274 | `Imp-688` | `modules::instance` | `src-tauri/src/modules/instance/caller_process_ancestry_tree_scanner.rs` | `src-tauri/src/modules/instance/tests_caller_process_ancestry_tree_scanner.rs` |
| Step 275 | `Imp-689` | `modules::release_gate` | `src-tauri/src/modules/release_gate/github_release_notes_parity_guard.rs` | `src-tauri/src/modules/release_gate/tests_github_release_notes_parity_guard.rs` |

---

## 4. Comprehensive End-to-End Test Scenarios

### Test Case `E2E-212-01`: Pipeline Response Header Sanitizer
- **Target Module**: `src-tauri/src/proxy/handlers/common/pipeline_response_header_sanitizer.rs`
- **Unit Suite**: `src-tauri/src/proxy/handlers/common/tests_pipeline_response_header_sanitizer.rs`
- **Improvement ID**: `Imp-685` (Step 271)
- **Invariant Addressed**: Pipeline First & Header Sanitization (`AGENTS.md`)
- **Preconditions**:
  - Raw upstream response headers received from an external AI provider (e.g., Google Gemini, OpenAI, Claude).
- **Actions / Execution Steps**:
  1. Pass upstream header map containing hop-by-hop headers (`connection: close`, `transfer-encoding: chunked`, `upgrade: ...`).
  2. Pass upstream header map containing leaking internal headers (`server: cloudflare`, `alt-svc: h3`, `x-goog-safety-ratings: ...`, `set-cookie: auth=xyz`).
  3. Invoke `sanitize_pipeline_response_headers(&raw_headers)` to clean and backfill headers.
  4. Verify normalized standard Antigravity headers (`content-type: text/event-stream; charset=utf-8`, `cache-control: no-cache`, `connection: keep-alive`).
- **Expected Outcomes**:
  - Leaking and hop-by-hop headers are completely stripped.
  - Required streaming headers are present and properly formatted.
  - Client connections receive only normalized Antigravity protocol headers.

---

### Test Case `E2E-212-02`: Window Control Capsule Formatter
- **Target Module**: `src-tauri/src/modules/instance/window_control_capsule_formatter.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_window_control_capsule_formatter.rs`
- **Improvement ID**: `Imp-686` (Step 272)
- **Invariant Addressed**: Minimalist & Contextual UI (Segmented Capsule Invariant) (`AGENTS.md`)
- **Preconditions**:
  - Desktop window actions (Minimize, Maximize, Close) ready for toolbar rendering.
- **Actions / Execution Steps**:
  1. Format individual window control buttons without grouping; assert validation warning against loose circular buttons.
  2. Ingest list of adjacent window control actions and format into unified segmented capsule.
  3. Verify applied CSS/Tailwind classes: `rounded-full`, shared border, subtle divider lines, and dark-glass backdrop.
  4. Verify nested `<select>` dropdown styles use borderless transparent styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to prevent pill-in-pill visual artifacts.
- **Expected Outcomes**:
  - Window controls render as a contiguous, cohesive dark-glass pill capsule.
  - Zero disjointed circular buttons or nested border visual defects.

---

### Test Case `E2E-212-03`: FIFO Dead-Letter Queue Spillover Guard
- **Target Module**: `src-tauri/src/modules/db_orm/fifo_dead_letter_queue_spillover_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/db_orm/tests_fifo_dead_letter_queue_spillover_guard.rs`
- **Improvement ID**: `Imp-687` (Step 273)
- **Invariant Addressed**: FIFO Restore & Queue Invariant (`AGENTS.md`)
- **Preconditions**:
  - Prompts failing repeatedly (exceeding retry threshold) or suffering unrecoverable syntax/upstream rejection errors.
- **Actions / Execution Steps**:
  1. Submit queries with `ORDER BY created_at ASC, id ASC`; verify `validate_spillover_query` returns `Ok(())`.
  2. Submit queries with `ORDER BY updated_at DESC` or `ORDER BY created_at DESC`; verify rejection with `InvertedSpilloverOrdering`.
  3. Submit queries lacking `id ASC` tie-breaker; verify rejection with `MissingIdTieBreaker`.
  4. Submit queries missing `ORDER BY created_at` or without `ORDER BY`; verify rejection with `MissingOriginalTimestamp`.
  5. Audit batches of dead-letter records via `record_dead_letter_spillover`:
     - Strictly increasing `(original_created_at, id)`: compliant, 0 violations.
     - Equal timestamps with increasing IDs: compliant tie-breaking.
     - Timestamp regression or descending IDs: flags `InvertedSpilloverOrdering` or `MissingIdTieBreaker`.
     - Missing `original_created_at`: flags `MissingOriginalTimestamp`.
- **Expected Outcomes**:
  - Full adherence to strict FIFO ordering during dead-letter quarantine transitions.
  - Full preservation of original timestamps and chronological lineage.

---

### Test Case `E2E-212-04`: Caller Process Ancestry Tree Scanner
- **Target Module**: `src-tauri/src/modules/instance/caller_process_ancestry_tree_scanner.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_caller_process_ancestry_tree_scanner.rs`
- **Improvement ID**: `Imp-688` (Step 274)
- **Invariant Addressed**: Caller IDE Self-Preservation (`AGENTS.md`)
- **Preconditions**:
  - Process tree snapshot with various parent/child relationships between IDEs, shells, CLI processes, and daemons.
- **Actions / Execution Steps**:
  1. Feed process tree where current CLI process is a descendant of `code.exe`, `cursor.exe`, or `windsurf.exe`.
  2. Scan ancestry upward to root; identify the active caller IDE process ID.
  3. Attempt instance stop/restart targeting the detected caller process; verify rejection with exit code 2 unless `--force` is provided.
  4. Feed process tree where current process is detached (no IDE parent); verify scan returns `None` and allows standard operations.
- **Expected Outcomes**:
  - Active developer IDE sessions are protected from accidental termination.
  - CLI operations maintain safety without developer intervention.

---

### Test Case `E2E-212-05`: GitHub Release Notes Parity Guard
- **Target Module**: `src-tauri/src/modules/release_gate/github_release_notes_parity_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_github_release_notes_parity_guard.rs`
- **Improvement ID**: `Imp-689` (Step 275)
- **Invariant Addressed**: Documentation & Attribution, Strictly Relative Paths (`AGENTS.md`)
- **Preconditions**:
  - Candidate GitHub release notes and `CHANGELOG.md` for release tag verification.
- **Actions / Execution Steps**:
  1. Pass release notes attributing exclusively `@aukgit`; assert parity pass.
  2. Pass release notes containing third-party user handles (`@contributor`, `@someone`); assert rejection with unauthorized handle violation.
  3. Pass release notes containing absolute filesystem paths (`D:\work\...`, `C:\Users\...`, `/var/log/...`); assert rejection with absolute path violation.
  4. Compare release notes heading and content against corresponding `CHANGELOG.md` version tag section; assert exact alignment.
- **Expected Outcomes**:
  - Release notes maintain strict attribution discipline and 100% relative path hygiene.
  - Release gating guarantees zero drift between repository changelogs and published GitHub releases.

---

## 5. Exact Targeted Deferred Test Commands
When an explicit testing prompt is issued, run the following commands sequentially:

```bash
# Step 271 (Imp-685): Pipeline Response Header Sanitizer
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_pipeline_response_header_sanitizer

# Step 272 (Imp-686): Window Control Capsule Formatter
cargo test --package antigravity-manager --lib modules::instance::tests_window_control_capsule_formatter

# Step 273 (Imp-687): FIFO Dead Letter Queue Spillover Guard
cargo test --package antigravity-manager --lib modules::db_orm::tests_fifo_dead_letter_queue_spillover_guard

# Step 274 (Imp-688): Caller Process Ancestry Tree Scanner
cargo test --package antigravity-manager --lib modules::instance::tests_caller_process_ancestry_tree_scanner

# Step 275 (Imp-689): GitHub Release Notes Parity Guard
cargo test --package antigravity-manager --lib modules::release_gate::tests_github_release_notes_parity_guard
```

---

## 6. Verification Status & Deferred Gate Protocol
- **Implementation Status**: Code authoring and unit test creation completed across all 5 tasks.
- **Verification Gate**: Execution deferred until the explicit testing phase per the Zero-Build & Zero-Test Invariant.
- **Module Index Registration**: Disjoint module registration is managed exclusively by the Lead Orchestrator.

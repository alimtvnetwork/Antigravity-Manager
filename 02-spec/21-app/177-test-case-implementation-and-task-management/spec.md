# Specification: Test Case Implementation and Task Management (Sequence 177)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `177-test-case-implementation-and-task-management`
- **Sequence**: 177
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 177th cycle of engineering execution in Antigravity-Manager, continuing the disciplined 10-step iteration rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize candidate eligibility boolean checks in `auto_switcher/scoring.rs` with an affirmative predicate (`is_candidate_eligible_for_switch`).
   - Invert nested condition trees in `proxy/handlers/claude/stream.rs` into early-return guard clauses.
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_keyring_mock_writer.rs`
   - `src-tauri/src/modules/instance/tests_session_file_reclaimer.rs`
   - `src-tauri/src/modules/instance/tests_port_collision_evictor.rs`
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-541: Reverse proxy SSE frame delimiter normalizer (`proxy/handlers/common/sse_frame_delimiter.rs` & `tests_sse_frame_delimiter.rs`).
   - Imp-542: Reverse proxy client request cancellation watcher (`proxy/handlers/common/request_cancellation_watcher.rs` & `tests_request_cancellation_watcher.rs`).
   - Imp-543: Auto-switcher candidate quota recovery forecaster (`auto_switcher/quota_recovery_forecaster.rs` & `tests_quota_recovery_forecaster.rs`).
   - Imp-544: Split-SQLite WAL page fragmentation surveyor (`db_orm/wal_fragmentation_surveyor.rs` & `tests_wal_fragmentation_surveyor.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-177-01` to `E2E-177-10`) in `02-spec/21-app/177-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**225 remaining steps**).

---

## 2. Sequence 177: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Auto-Switcher Candidate Scoring
- **Target File**: `src-tauri/src/modules/auto_switcher/scoring.rs`
- **Scope**: Extract affirmative predicate `is_candidate_eligible_for_switch` to replace compound negative conditionals and nested filters.

### Step 2: Guard Clause Inversion in Claude Stream Chunk Processor
- **Target File**: `src-tauri/src/proxy/handlers/claude/stream.rs`
- **Scope**: Flatten event stream chunk parsing and error checking loops with discrete early-return guard clauses.

### Step 3: Multi-Instance Mock Keyring Writer Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_keyring_mock_writer.rs`
- **Scope**: Validate mock credentials generation, bypass marker file creation, and directory permission integrity.

### Step 4: Multi-Instance Volatile Session File Reclaimer Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_session_file_reclaimer.rs`
- **Scope**: Validate volatile session cleanup, PID directories, and stale lockfile sweeps.

### Step 5: Multi-Instance Port Collision Evictor Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_port_collision_evictor.rs`
- **Scope**: Validate TCP socket conflict checks, range bounds [9222..9322], and collision-free port allocation.

### Step 6: Reverse Proxy SSE Frame Delimiter Normalizer (Imp-541)
- **Target Files**:
  - `src-tauri/src/proxy/handlers/common/sse_frame_delimiter.rs`
  - `src-tauri/src/proxy/handlers/common/tests_sse_frame_delimiter.rs`
- **Scope**: Normalize SSE frame endings to canonical `\n\n`, convert CRLF/LF variations, and handle fragmented chunks.

### Step 7: Reverse Proxy Client Request Cancellation Watcher (Imp-542)
- **Target Files**:
  - `src-tauri/src/proxy/handlers/common/request_cancellation_watcher.rs`
  - `src-tauri/src/proxy/handlers/common/tests_request_cancellation_watcher.rs`
- **Scope**: Wrap atomic cancellation token to detect client stream aborts and avoid upstream resource leaks.

### Step 8: Auto-Switcher Candidate Quota Recovery Forecaster (Imp-543)
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/quota_recovery_forecaster.rs`
  - `src-tauri/src/modules/auto_switcher/tests_quota_recovery_forecaster.rs`
- **Scope**: Predict quota replenishment time windows given rolling reset epochs and current velocity.

### Step 9: Split-SQLite WAL Page Fragmentation Surveyor (Imp-544)
- **Target Files**:
  - `src-tauri/src/modules/db_orm/wal_fragmentation_surveyor.rs`
  - `src-tauri/src/modules/db_orm/tests_wal_fragmentation_surveyor.rs`
- **Scope**: Inspect WAL size, uncheckpointed page count, and fragmentation percentage to recommend passive vs truncate checkpoints.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/177-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/177-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-177-01` to `E2E-177-10`) and update pending task backlog (225 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

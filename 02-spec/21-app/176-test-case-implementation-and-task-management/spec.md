# Specification: Test Case Implementation and Task Management (Sequence 176)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `176-test-case-implementation-and-task-management`
- **Sequence**: 176
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 176th cycle of engineering execution in Antigravity-Manager, continuing the 10-step iteration rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**: Refactoring compound negative expressions in `instance/lifecycle.rs` into affirmative predicates (`is_instance_running_and_active`) and inverting nested condition trees in `repo_db/discovery.rs` into early-return guard clauses.
2. **Unit Test Suite Implementation (Steps 3 through 9)**: Authoring concrete, exhaustive unit tests for recently added modules:
   - Multi-Instance process management: `stale_pid_reaper.rs`, `profile_state_hasher.rs`, `binary_execution_probe.rs`.
   - Auto-Switcher candidate scoring: `account_staleness_detector.rs`, `headroom_score_multiplier.rs`, `headroom_trend.rs`.
   - Split-SQLite maintenance: `page_freelist_analyzer.rs`, `connection_busy_timeout_scaler.rs`.
3. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**: Cataloging deferred E2E tests in `02-spec/21-app/176-test-case-implementation-and-task-management/e2e_master_plan.md` and recalculating the remaining backlog across all tracks (**235 remaining steps**).

---

## 2. Sequence 176: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Instance Lifecycle
- **Target File**: `src-tauri/src/modules/instance/lifecycle.rs`
- **Scope**: Replace compound negative conjunctions with affirmative helper functions (`is_instance_running_and_active`).

### Step 2: Guard Clause Inversion in Running Prompt Discovery
- **Target File**: `src-tauri/src/modules/repo_db/discovery.rs`
- **Scope**: Flatten transcript scanning loop into discrete early-return guard clauses, eliminating nested `if` blocks.

### Step 3: Stale Instance PID Process Reaper Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_stale_pid_reaper.rs`
- **Scope**: Unit tests for `stale_pid_reaper.rs` covering alive PID retention, dead PID reaping, and status transitions.

### Step 4: Instance Profile Configuration SHA-256 Hasher Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_profile_hasher.rs`
- **Scope**: Unit tests for `profile_state_hasher.rs` covering state hashing, drift detection, and deterministic JSON hashing.

### Step 5: IDE Binary Architecture and Execution Probe Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_binary_execution_probe.rs`
- **Scope**: Unit tests for `binary_execution_probe.rs` covering executable validation, header checks, and missing binary rejection.

### Step 6: Inactive Account Staleness Detector Unit Test Suite
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_account_staleness.rs`
- **Scope**: Unit tests for `account_staleness_detector.rs` covering epoch checks, staleness thresholds, and candidate probing.

### Step 7: Candidate Headroom Ratio & Multiplier Unit Test Suite
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_headroom_multiplier.rs`
- **Scope**: Unit tests for `headroom_score_multiplier.rs` and `headroom_trend.rs` covering headroom ratio scaling and zero quota gates.

### Step 8: Split-SQLite Page Freelist Reclaim Analyzer Unit Test Suite
- **Target File**: `src-tauri/src/modules/db_orm/tests_page_freelist.rs`
- **Scope**: Unit tests for `page_freelist_analyzer.rs` covering freelist page counting and incremental vacuum recommendations.

### Step 9: Database Connection Busy Timeout Scaler Unit Test Suite
- **Target File**: `src-tauri/src/modules/db_orm/tests_busy_timeout_scaler.rs`
- **Scope**: Unit tests for `connection_busy_timeout_scaler.rs` covering adaptive busy timeout scaling under connection load.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/176-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/176-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-176-01` to `E2E-176-10`) and update pending task backlog (235 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

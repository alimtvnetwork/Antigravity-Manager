# Specification: Test Case Implementation and Task Management (Sequence 183)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `183-test-case-implementation-and-task-management`
- **Sequence**: 183
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 183rd cycle of engineering execution in Antigravity-Manager, maintaining the disciplined 10-step rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize stop sequence matching in `src-tauri/src/proxy/handlers/claude/stop_sequence_matcher.rs` with affirmative helper predicates (`is_stop_sequence_matched`, `is_partial_match`, `is_exact_suffix_match`).
   - Invert condition ladders in `src-tauri/src/proxy/handlers/claude/stop_reason_normalizer.rs` with affirmative helpers (`is_terminal_stop_reason`, `normalize_claude_stop_reason`) and early returns.
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_profile_dir_validator.rs`: Path existence verification, path traversal protection, and directory permissions.
   - `src-tauri/src/modules/instance/tests_workspace_dir_validator.rs`: Workspace folder boundary validation, directory auto-creation, and symlink escape defense.
   - `src-tauri/src/modules/instance/tests_sandboxed_storage_cleaner.rs`: Volatile cache purging (Code Cache, GPUCache, cache.nss) while preserving persistent settings.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-565: Reverse proxy streaming timeout sentinel unit test suite (`proxy/upstream/tests_streaming_timeout_sentinel.rs`).
   - Imp-566: Reverse proxy route latency calculator unit test suite (`proxy/upstream/tests_route_latency_calculator.rs`).
   - Imp-567: Auto-switcher quota acceleration detector unit test suite (`auto_switcher/tests_quota_acceleration_detector.rs`).
   - Imp-568: Split-SQLite connection checkout limiter unit test suite (`db_orm/tests_connection_checkout_limiter.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-183-01` to `E2E-183-10`) in `02-spec/21-app/183-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**165 remaining steps**).

---

## 2. Sequence 183: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Stop Sequence Matcher
- **Target File**: `src-tauri/src/proxy/handlers/claude/stop_sequence_matcher.rs`
- **Scope**: Extract affirmative helper predicates `is_stop_sequence_matched`, `is_partial_match`, and `is_exact_suffix_match`.

### Step 2: Guard Clause Inversion in Stop Reason Normalizer
- **Target File**: `src-tauri/src/proxy/handlers/claude/stop_reason_normalizer.rs`
- **Scope**: Flatten chained condition ladders with discrete early-return helper `is_terminal_stop_reason`.

### Step 3: Multi-Instance Profile Dir Validator Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_profile_dir_validator.rs`
- **Scope**: Validate instance profile root paths, traversal defense, and directory structure.

### Step 4: Multi-Instance Workspace Dir Validator Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_workspace_dir_validator.rs`
- **Scope**: Validate workspace boundary integrity, auto-creation of missing roots, and symlink escape defenses.

### Step 5: Multi-Instance Sandboxed Storage Cleaner Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_sandboxed_storage_cleaner.rs`
- **Scope**: Validate purging of volatile runtime caches while strictly maintaining `settings.json`, `keybindings.json`, and SQLite state databases.

### Step 6: Reverse Proxy Upstream Streaming Timeout Sentinel Unit Test Suite (Imp-565)
- **Target File**: `src-tauri/src/proxy/upstream/tests_streaming_timeout_sentinel.rs`
- **Scope**: Validate idle chunk timeouts, active streaming grace periods, and timeout cancellation on EOF.

### Step 7: Reverse Proxy Upstream Route Latency Calculator Unit Test Suite (Imp-566)
- **Target File**: `src-tauri/src/proxy/upstream/tests_route_latency_calculator.rs`
- **Scope**: Validate percentile calculations (p50, p90, p99), outlier elimination, and composite score weighting.

### Step 8: Auto-Switcher Quota Acceleration Detector Unit Test Suite (Imp-567)
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_quota_acceleration_detector.rs`
- **Scope**: Validate second-derivative acceleration detection, rapid depletion alarms, and cooldown preemption.

### Step 9: Split-SQLite Connection Checkout Limiter Unit Test Suite (Imp-568)
- **Target File**: `src-tauri/src/modules/db_orm/tests_connection_checkout_limiter.rs`
- **Scope**: Validate concurrent checkout quotas, semaphore permit replenishment, and timeout on saturation.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/183-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/183-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-183-01` to `E2E-183-10`) and update pending task backlog (165 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

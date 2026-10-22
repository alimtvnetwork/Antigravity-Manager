# Specification: Test Case Implementation and Task Management (Sequence 182)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `182-test-case-implementation-and-task-management`
- **Sequence**: 182
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 182nd cycle of engineering execution in Antigravity-Manager, maintaining the disciplined 10-step rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize message turn validation in `src-tauri/src/proxy/handlers/claude/message_turn_validator.rs` with affirmative helper predicates (`is_valid_turn_sequence`, `is_alternating_role_flow`, `is_terminal_turn`).
   - Invert nested condition trees and string stripping in `src-tauri/src/proxy/handlers/claude/role_prefix_sanitizer.rs` with discrete early-return guard clauses and affirmative helpers (`strip_leading_role_markers`).
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_child_termination_guard.rs`: Grace period timeout, multi-stage fallback ladder, and process tree termination.
   - `src-tauri/src/modules/instance/tests_process_vitality_probe.rs`: High-performance PID vitality caching, polling debounce, and dead PID detection.
   - `src-tauri/src/modules/instance/tests_ipc_socket_cleaner.rs`: Stale Unix domain sockets, Windows named pipes sweep, and file lock reclamation.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-561: Reverse proxy endpoint health decay unit test suite (`proxy/upstream/tests_endpoint_health_decay.rs`).
   - Imp-562: Reverse proxy endpoint weight matrix unit test suite (`proxy/upstream/tests_endpoint_weight_matrix.rs`).
   - Imp-563: Auto-switcher dynamic quota floor unit test suite (`auto_switcher/tests_dynamic_quota_floor.rs`).
   - Imp-564: Split-SQLite page budget controller unit test suite (`db_orm/tests_page_budget_controller.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-182-01` to `E2E-182-10`) in `02-spec/21-app/182-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**175 remaining steps**).

---

## 2. Sequence 182: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Message Turn Validator
- **Target File**: `src-tauri/src/proxy/handlers/claude/message_turn_validator.rs`
- **Scope**: Extract affirmative helper predicates `is_valid_turn_sequence`, `is_alternating_role_flow`, and `is_terminal_turn`.

### Step 2: Guard Clause Inversion in Role Prefix Sanitizer
- **Target File**: `src-tauri/src/proxy/handlers/claude/role_prefix_sanitizer.rs`
- **Scope**: Flatten chained condition ladders with discrete early-return helper `strip_leading_role_markers`.

### Step 3: Multi-Instance Child Termination Guard Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_child_termination_guard.rs`
- **Scope**: Validate grace period countdown, multi-stage escalation ladder (SIGTERM -> SIGKILL), and orphan avoidance.

### Step 4: Multi-Instance Process Vitality Probe Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_process_vitality_probe.rs`
- **Scope**: Validate TTL-cached vitality checks, dead process detection, and batch status extraction.

### Step 5: Multi-Instance IPC Socket Cleaner Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_ipc_socket_cleaner.rs`
- **Scope**: Validate stale socket discovery, IPC handle cleanup, and error-tolerant sweeping.

### Step 6: Reverse Proxy Upstream Endpoint Health Decay Unit Test Suite (Imp-561)
- **Target File**: `src-tauri/src/proxy/upstream/tests_endpoint_health_decay.rs`
- **Scope**: Validate exponential decay factor, half-life duration, and recovery score restoration.

### Step 7: Reverse Proxy Upstream Endpoint Weight Matrix Unit Test Suite (Imp-562)
- **Target File**: `src-tauri/src/proxy/upstream/tests_endpoint_weight_matrix.rs`
- **Scope**: Validate composite weight scoring (latency + error rate + quota), route selection, and tie-breaking.

### Step 8: Auto-Switcher Dynamic Quota Floor Unit Test Suite (Imp-563)
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_dynamic_quota_floor.rs`
- **Scope**: Validate dynamic floor adjustments under high burst load, floor clamping [5.0..25.0], and model-specific floors.

### Step 9: Split-SQLite Page Budget Controller Unit Test Suite (Imp-564)
- **Target File**: `src-tauri/src/modules/db_orm/tests_page_budget_controller.rs`
- **Scope**: Validate memory budget clamping across multi-instance SQLite connections, soft cap enforcement, and eviction triggers.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/182-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/182-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-182-01` to `E2E-182-10`) and update pending task backlog (175 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

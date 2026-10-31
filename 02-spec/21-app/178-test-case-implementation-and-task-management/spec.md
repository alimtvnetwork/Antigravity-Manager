# Specification: Test Case Implementation and Task Management (Sequence 178)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `178-test-case-implementation-and-task-management`
- **Sequence**: 178
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 178th cycle of engineering execution in Antigravity-Manager, maintaining the disciplined 10-step rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize node identifier recognition in `src-tauri/src/modules/telegram_inbound/processing.rs` with affirmative helper predicates (`is_valid_node_identifier`, `is_command_target_local`).
   - Invert nested condition trees and heavy rate-limiting branches in `src-tauri/src/modules/email_inbound/dispatcher.rs` with discrete early-return guard clauses and affirmative classification (`is_heavy_action_type`).
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_cloned_guid_reseeder.rs`: Synthetic UUID format, storage.json key injection, and collision avoidance.
   - `src-tauri/src/modules/instance/tests_cloned_telemetry_salt.rs`: Profile ID salting, hash-based GUID generation, and sqmId casing.
   - `src-tauri/src/modules/instance/tests_env_token_scrubber.rs`: Environment token scrubbing, sensitive key allowlist/blocklist, and case-insensitive matching.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-545: Reverse proxy upstream pool connection health probe (`proxy/upstream/connection_health_probe.rs` & `tests_connection_health_probe.rs`).
   - Imp-546: Reverse proxy HTTP/2 keepalive ping scheduler (`proxy/upstream/h2_ping_scheduler.rs` & `tests_h2_ping_scheduler.rs`).
   - Imp-547: Auto-switcher rapid quota drain circuit breaker (`auto_switcher/drain_circuit_breaker.rs` & `tests_drain_circuit_breaker.rs`).
   - Imp-548: Split-SQLite connection lock contention watchdog (`db_orm/lock_contention_watchdog.rs` & `tests_lock_contention_watchdog.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-178-01` to `E2E-178-10`) in `02-spec/21-app/178-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**215 remaining steps**).

---

## 2. Sequence 178: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Telegram Inbound Processing
- **Target File**: `src-tauri/src/modules/telegram_inbound/processing.rs`
- **Scope**: Extract affirmative helper predicates `is_valid_node_identifier` and `is_command_target_local` to eliminate compound negative logic and nested whitespace checks.

### Step 2: Guard Clause Inversion in Email Inbound Action Dispatcher
- **Target File**: `src-tauri/src/modules/email_inbound/dispatcher.rs`
- **Scope**: Invert nested rate-limit checks into early-return guard clauses and extract `is_heavy_action_type`.

### Step 3: Multi-Instance Cloned GUID Reseeder Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_cloned_guid_reseeder.rs`
- **Scope**: Validate UUID format adherence, injection across all 5 target telemetry keys, and non-destructive JSON modification.

### Step 4: Multi-Instance Cloned Telemetry Salt Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_cloned_telemetry_salt.rs`
- **Scope**: Validate isolated salt generation, deterministic hashing format, and uppercase braced sqmId syntax.

### Step 5: Multi-Instance Environment Token Scrubber Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_env_token_scrubber.rs`
- **Scope**: Validate scrubbing of known API keys, case-insensitivity, count accuracy, and benign variable retention.

### Step 6: Reverse Proxy Upstream Connection Health Probe (Imp-545)
- **Target Files**:
  - `src-tauri/src/proxy/upstream/connection_health_probe.rs`
  - `src-tauri/src/proxy/upstream/tests_connection_health_probe.rs`
- **Scope**: Monitor upstream connection latency, consecutive failures, and state transitions (`Healthy`, `Degraded`, `Dead`).

### Step 7: Reverse Proxy HTTP/2 Keepalive Ping Scheduler (Imp-546)
- **Target Files**:
  - `src-tauri/src/proxy/upstream/h2_ping_scheduler.rs`
  - `src-tauri/src/proxy/upstream/tests_h2_ping_scheduler.rs`
- **Scope**: Schedule periodic HTTP/2 PING frames to prevent silent middlebox connection drops during idle periods.

### Step 8: Auto-Switcher Rapid Quota Drain Circuit Breaker (Imp-547)
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/drain_circuit_breaker.rs`
  - `src-tauri/src/modules/auto_switcher/tests_drain_circuit_breaker.rs`
- **Scope**: Detect sudden abnormal quota drops (>25% within 60s) and trip circuit breaker to protect account from burn.

### Step 9: Split-SQLite Connection Lock Contention Watchdog (Imp-548)
- **Target Files**:
  - `src-tauri/src/modules/db_orm/lock_contention_watchdog.rs`
  - `src-tauri/src/modules/db_orm/tests_lock_contention_watchdog.rs`
- **Scope**: Track active connection lock duration, detect contention exceeding threshold, and trigger lock release recommendation.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/178-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/178-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-178-01` to `E2E-178-10`) and update pending task backlog (215 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

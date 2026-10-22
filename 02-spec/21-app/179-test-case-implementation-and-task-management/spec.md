# Specification: Test Case Implementation and Task Management (Sequence 179)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `179-test-case-implementation-and-task-management`
- **Sequence**: 179
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 179th cycle of engineering execution in Antigravity-Manager, continuing the disciplined 10-step iteration rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize proxy candidate qualification in `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs` with affirmative helper predicates (`is_proxy_eligible_for_shared_pool`, `is_bound_proxy_eligible`).
   - Invert nested condition trees and telemetry parsing loops in `src-tauri/src/modules/email_inbound/imap.rs` with discrete early-continue guard clauses and affirmative helpers (`is_telemetry_from_remote_machine`, `is_telemetry_event_active`).
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_crash_loop_backoff.rs`: Exponential relaunch backoff delays (2s..30s), short lifespan threshold (<10s), and counter resets on healthy runs.
   - `src-tauri/src/modules/instance/tests_crash_loop_guard.rs`: Rapid crash quarantine thresholds (3 rapid crashes < 5s), 120s quarantine window, and thread-safe record management.
   - `src-tauri/src/modules/instance/tests_child_health_monitor.rs`: Heartbeat silence detection (>45s stall threshold), heartbeat pulse tracking, and clean deregistration.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-549: Reverse proxy dynamic route balancer unit test suite (`proxy/upstream/tests_dynamic_route_balancer.rs`).
   - Imp-550: Reverse proxy upstream TLS SNI sanitizer unit test suite (`proxy/upstream/tests_tls_sni_sanitizer.rs`).
   - Imp-551: Auto-switcher quota reset epoch drift compensator (`auto_switcher/epoch_drift_compensator.rs` & `tests_epoch_drift_compensator.rs`).
   - Imp-552: Split-SQLite automatic index fragmentation scorer (`db_orm/index_fragmentation_scorer.rs` & `tests_index_fragmentation_scorer.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-179-01` to `E2E-179-10`) in `02-spec/21-app/179-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**205 remaining steps**).

---

## 2. Sequence 179: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Proxy Pool Candidate Routing
- **Target File**: `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs`
- **Scope**: Extract affirmative helper predicates `is_proxy_eligible_for_shared_pool` and `is_bound_proxy_eligible` to eliminate compound negative filter chains.

### Step 2: Guard Clause Inversion in Email Inbound IMAP Telemetry Processing
- **Target File**: `src-tauri/src/modules/email_inbound/imap.rs`
- **Scope**: Invert nested telemetry evaluation loops with early-continue guard clauses and extract `is_telemetry_from_remote_machine` and `is_telemetry_event_active`.

### Step 3: Multi-Instance Crash Loop Backoff Guard Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_crash_loop_backoff.rs`
- **Scope**: Validate exponential backoff progression, short-runtime crash detection (<10s), healthy run resets, and max backoff clamps (30s).

### Step 4: Multi-Instance Rapid Crash Quarantine Guard Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_crash_loop_guard.rs`
- **Scope**: Validate rapid crash quarantine trigger (3 rapid crashes < 5s), 120s quarantine expiration, and manual reset.

### Step 5: Multi-Instance Child Process Health Heartbeat Monitor Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_child_health_monitor.rs`
- **Scope**: Validate heartbeat arrival registration, silence duration tracking, stall threshold evaluation (>45s), and instance removal.

### Step 6: Reverse Proxy Dynamic Route Balancer Unit Test Suite (Imp-549)
- **Target File**: `src-tauri/src/proxy/upstream/tests_dynamic_route_balancer.rs`
- **Scope**: Validate route registration, weight reduction on failure, gradual recovery on success, and minimum weight floors (5).

### Step 7: Reverse Proxy Upstream TLS SNI Sanitizer Unit Test Suite (Imp-550)
- **Target File**: `src-tauri/src/proxy/upstream/tests_tls_sni_sanitizer.rs`
- **Scope**: Validate scheme stripping, port extraction, FQDN length bounds, IDN/Punycode detection, and IPv4/IPv6 literal normalization.

### Step 8: Auto-Switcher Quota Reset Epoch Drift Compensator (Imp-551)
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/epoch_drift_compensator.rs`
  - `src-tauri/src/modules/auto_switcher/tests_epoch_drift_compensator.rs`
- **Scope**: Detect clock drift between local system time and upstream quota reset epochs, applying sliding offsets to prevent premature switch triggers.

### Step 9: Split-SQLite Automatic Index Fragmentation Scorer (Imp-552)
- **Target Files**:
  - `src-tauri/src/modules/db_orm/index_fragmentation_scorer.rs`
  - `src-tauri/src/modules/db_orm/tests_index_fragmentation_scorer.rs`
- **Scope**: Quantify B-Tree index fragmentation percentage and recommend `REINDEX` or `ANALYZE` operations for bloated table indices.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/179-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/179-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-179-01` to `E2E-179-10`) and update pending task backlog (205 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

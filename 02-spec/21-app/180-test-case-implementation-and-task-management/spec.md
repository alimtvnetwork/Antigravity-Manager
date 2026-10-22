# Specification: Test Case Implementation and Task Management (Sequence 180)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `180-test-case-implementation-and-task-management`
- **Sequence**: 180
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 180th cycle of engineering execution in Antigravity-Manager, maintaining the disciplined 10-step rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize proxy request header validation in `src-tauri/src/proxy/handlers/common/header_allowlist_filter.rs` with affirmative helper predicates (`is_hop_by_hop_header`, `is_forwardable_header`).
   - Invert nested condition trees and prefix matching ladders in `src-tauri/src/modules/email_inbound/parse_legacy.rs` with discrete early-return guard clauses and affirmative helpers (`extract_prompt_injection_prefix_len`, `extract_fast_forward_target`).
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_window_state_restorer.rs`: Default window bounds (1280x800), JSON persistence, and missing file fallback.
   - `src-tauri/src/modules/instance/tests_window_layout_anchor.rs`: Staggered window offset coordinates (40px stagger), modulo 8 wrapping, and custom base coordinates.
   - `src-tauri/src/modules/instance/tests_vitality_scanner.rs`: Process vitality checks, PID 0 guard, and batch PID verification.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-553: Reverse proxy transport metrics collector unit test suite (`proxy/upstream/tests_transport_metrics_collector.rs`).
   - Imp-554: Reverse proxy dynamic connection pool sizer unit test suite (`proxy/upstream/tests_dynamic_pool_sizer.rs`).
   - Imp-555: Auto-switcher hourly entropy and oscillation guard unit test suite (`auto_switcher/tests_hourly_entropy_guard.rs`).
   - Imp-556: Split-SQLite page cache adjuster unit test suite (`db_orm/tests_page_cache_adjuster.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-180-01` to `E2E-180-10`) in `02-spec/21-app/180-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**195 remaining steps**).

---

## 2. Sequence 180: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Header Allowlist Filter
- **Target File**: `src-tauri/src/proxy/handlers/common/header_allowlist_filter.rs`
- **Scope**: Extract affirmative helper predicates `is_hop_by_hop_header`, `is_client_trace_header`, and `is_forwardable_header`.

### Step 2: Guard Clause Inversion in Email Inbound Legacy Parsing
- **Target File**: `src-tauri/src/modules/email_inbound/parse_legacy.rs`
- **Scope**: Flatten chained prefix conditions with dedicated early-return helpers `extract_prompt_injection_prefix_len` and `extract_fast_forward_target`.

### Step 3: Multi-Instance Window State Restorer Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_window_state_restorer.rs`
- **Scope**: Validate default window dimensions, round-trip serialization/deserialization, and corrupted state resilience.

### Step 4: Multi-Instance Window Layout Coordinates Anchor Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_window_layout_anchor.rs`
- **Scope**: Validate staggered screen coordinates offset (40px per instance), 8-instance modulo wrap, and default coordinate anchors.

### Step 5: Multi-Instance Process Vitality Scanner Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_vitality_scanner.rs`
- **Scope**: Validate PID 0 rejection, current process vitality (`std::process::id()`), and batch PID vitality vector mappings.

### Step 6: Reverse Proxy Upstream Transport Metrics Collector Unit Test Suite (Imp-553)
- **Target File**: `src-tauri/src/proxy/upstream/tests_transport_metrics_collector.rs`
- **Scope**: Validate transport timing ingestion, exponential moving average TTFB calculations (80/20 weighting), and missing endpoint queries.

### Step 7: Reverse Proxy Dynamic Connection Pool Sizer Unit Test Suite (Imp-554)
- **Target File**: `src-tauri/src/proxy/upstream/tests_dynamic_pool_sizer.rs`
- **Scope**: Validate in-flight request tracking, dynamic allocation bounds [MIN_IDLE=5 .. MAX_IDLE=50], and saturating subtraction.

### Step 8: Auto-Switcher Hourly Entropy & Oscillation Guard Unit Test Suite (Imp-555)
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_hourly_entropy_guard.rs`
- **Scope**: Validate Shannon entropy calculation, single-target vs diverse-target entropy bounds, and oscillation storm detection (<0.35 threshold).

### Step 9: Split-SQLite Page Cache Memory Adjuster Unit Test Suite (Imp-556)
- **Target File**: `src-tauri/src/modules/db_orm/tests_page_cache_adjuster.rs`
- **Scope**: Validate negative PRAGMA cache_size syntax, KiB clamping [1024 .. 65536], and in-memory database configuration reading.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/180-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/180-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-180-01` to `E2E-180-10`) and update pending task backlog (195 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

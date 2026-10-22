# Specification: Test Case Implementation and Task Management (Sequence 181)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `181-test-case-implementation-and-task-management`
- **Sequence**: 181
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 181st cycle of engineering execution in Antigravity-Manager, maintaining the disciplined 10-step rhythm:
1. **Boolean Modernization & Guard Inversion (Steps 1 & 2)**:
   - Modernize Claude protocol message validation in `src-tauri/src/proxy/handlers/claude/messages.rs` with affirmative helper predicates (`is_valid_role`, `is_text_content_block`, `is_forwardable_claude_message`).
   - Invert nested condition trees and qualification checks in `src-tauri/src/modules/auto_switcher/scoring.rs` with discrete early-return guard clauses and affirmative helpers (`is_candidate_eligible_for_switch`).
2. **Multi-Instance Unit Test Suites (Steps 3, 4 & 5)**:
   - `src-tauri/src/modules/instance/tests_profile_fingerprint_hasher.rs`: SHA-256 fingerprint generation, canonical JSON serialization, and drift detection.
   - `src-tauri/src/modules/instance/tests_remote_debugging_port_allocator.rs`: Port range allocation (9222..9322), sequential slot discovery, and collision evasion.
   - `src-tauri/src/modules/instance/tests_session_trace_uuid.rs`: Session UUID v4 generation, entropy validation, and instance profile binding.
3. **Continuous Improvements & Test Suites (Steps 6, 7, 8 & 9)**:
   - Imp-557: Reverse proxy latency exponential smoother unit test suite (`proxy/upstream/tests_latency_exponential_smoother.rs`).
   - Imp-558: Reverse proxy HTTP/2 flow controller unit test suite (`proxy/upstream/tests_http2_flow_controller.rs`).
   - Imp-559: Auto-switcher score variance guard unit test suite (`auto_switcher/tests_score_variance_guard.rs`).
   - Imp-560: Split-SQLite MMAP tuner unit test suite (`db_orm/tests_mmap_tuner.rs`).
4. **End-to-End Test Enqueueing & Backlog Recalculation (Step 10)**:
   - Cataloging deferred E2E tests (`E2E-181-01` to `E2E-181-10`) in `02-spec/21-app/181-test-case-implementation-and-task-management/e2e_master_plan.md`.
   - Recalculating remaining backlog across all tracks (**185 remaining steps**).

---

## 2. Sequence 181: 10-Step Execution Breakdown

### Step 1: Boolean Logic Modernization in Claude Messages Handler
- **Target File**: `src-tauri/src/proxy/handlers/claude/messages.rs`
- **Scope**: Extract affirmative helper predicates `is_valid_role`, `is_text_content_block`, and `is_forwardable_claude_message`.

### Step 2: Guard Clause Inversion in Auto-Switcher Candidate Scoring
- **Target File**: `src-tauri/src/modules/auto_switcher/scoring.rs`
- **Scope**: Flatten nested qualification checks into affirmative `is_candidate_eligible_for_switch` and discrete early-return guards.

### Step 3: Multi-Instance Profile Fingerprint Hasher Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_profile_fingerprint_hasher.rs`
- **Scope**: Validate SHA-256 fingerprint stability, canonical JSON key sorting, and configuration drift detection.

### Step 4: Multi-Instance Remote Debugging Port Allocator Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_remote_debugging_port_allocator.rs`
- **Scope**: Validate port allocation within range [9222 .. 9322], sequential collision resistance, and wrap-around.

### Step 5: Multi-Instance Session Trace UUID Unit Test Suite
- **Target File**: `src-tauri/src/modules/instance/tests_session_trace_uuid.rs`
- **Scope**: Validate UUID v4 formatting (36 characters), randomness entropy, and per-launch uniqueness.

### Step 6: Reverse Proxy Upstream Latency Exponential Smoother Unit Test Suite (Imp-557)
- **Target File**: `src-tauri/src/proxy/upstream/tests_latency_exponential_smoother.rs`
- **Scope**: Validate exponential moving average calculations, alpha weighting factor, and spike damping.

### Step 7: Reverse Proxy HTTP/2 Flow Controller Unit Test Suite (Imp-558)
- **Target File**: `src-tauri/src/proxy/upstream/tests_http2_flow_controller.rs`
- **Scope**: Validate initial window size, stream buffer throttling, and flow-control credit replenishment.

### Step 8: Auto-Switcher Score Variance Guard Unit Test Suite (Imp-559)
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_score_variance_guard.rs`
- **Scope**: Validate variance calculation across candidate scores, outlier dampening, and selection stability.

### Step 9: Split-SQLite MMAP Memory Tuner Unit Test Suite (Imp-560)
- **Target File**: `src-tauri/src/modules/db_orm/tests_mmap_tuner.rs`
- **Scope**: Validate PRAGMA mmap_size syntax, platform memory clamping, and in-memory verification.

### Step 10: Master End-to-End Test Plan Update & Backlog Enqueueing
- **Target Files**:
  - `02-spec/21-app/181-test-case-implementation-and-task-management/e2e_master_plan.md`
  - `.ai-memory/plans/completed/181-test-case-implementation-and-task-management.md`
- **Scope**: Catalog 10 deferred integration test cases (`E2E-181-01` to `E2E-181-10`) and update pending task backlog (185 remaining steps).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

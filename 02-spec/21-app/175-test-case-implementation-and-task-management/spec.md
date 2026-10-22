# Specification: Test Case Implementation and Task Management (Sequence 175)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `175-test-case-implementation-and-task-management`
- **Sequence**: 175
- **Date**: 2026-10-23
- **Author**: Lead Orchestrator
- **Constraint Policy**: Strictly zero build execution, zero test runner execution during implementation phase; execute 10 steps per cycle; enqueue comprehensive end-to-end test execution plans for subsequent AI verification; strictly relative repository paths.

---

## 1. Executive Summary & Architectural Intent

This specification establishes the 175th cycle of engineering execution in Antigravity-Manager, focusing on:
1. **Unit Test Suite Implementation**: Writing robust, exhaustive unit test assertions for recent architectural improvements (Improvements 481–540 implemented in Sequences 173 and 174) covering the reverse proxy engine, auto-switcher quota guards, Split-SQLite maintenance, and sandboxed multi-instance isolation.
2. **Pending Task Inventory & Step Calculation**: Analyzing all active tracks across the repository to determine exact step requirements:
   - **Track A (Boolean Logic Modernization & Guard Clauses)**: 15 steps
   - **Track B (Multi-Instance Isolation & Prompt Resumption E2E Testing)**: 25 steps
   - **Track C (UI & Audit Trail Modernization)**: 20 steps
   - **Track D (Release Gates & Installer Verification)**: 15 steps
   - **Track E (Continuous Codebase Improvements 541–720)**: 180 steps (6 batches of 30)
   - **Total Pending Steps**: **255 steps** total (245 remaining after the 10 steps executed in Sequence 175).
3. **Execution Rhythm**: Executing precisely **10 steps per turn**, reporting completed items, and detailing the pending queue at the conclusion of each turn.
4. **End-to-End Test Enqueueing**: Writing an exhaustive End-to-End test execution plan and test case registry in `.ai-memory/plans/pending/175-test-case-implementation-and-task-management.md` and `.ai-memory/plans/subtasks/175-test-case-implementation-and-task-management/` for future execution when specifically commanded by the user.

---

## 2. Sequence 175: 10-Step Execution Breakdown

### Step 1: Unit Test Suite for SSE Stream Splitter & Preamble Padder
- **Target File**: `src-tauri/src/proxy/handlers/common/tests_sse_splitter_padder.rs`
- **Coverage**:
  - `sse_stream_splitter.rs` (`split_sse_stream_frames`, frame delimiter handling `\r\n\r\n` vs `\n\n`, partial frame buffering, UTF-8 boundary preservation).
  - `sse_stream_padder.rs` (`pad_sse_stream_preamble`, initial byte padding for client proxy buffer flushing, zero-content event handling).

### Step 2: Unit Test Suite for Upstream Connection Backoff Matrix & TCP Keepalive Probe
- **Target File**: `src-tauri/src/proxy/upstream/tests_backoff_and_probe.rs`
- **Coverage**:
  - `connection_backoff_matrix.rs` (exponential backoff calculations, jitter distribution, max ceiling bounds, retry attempt tracking).
  - `tcp_keepalive_probe.rs` (keepalive duration configuration, probe idle intervals, TCP timeout thresholds).

### Step 3: Unit Test Suite for Request Gzip Compression Sniffer
- **Target File**: `src-tauri/src/proxy/handlers/common/tests_request_gzip_sniffer.rs`
- **Coverage**:
  - `request_gzip_sniffer.rs` (magic byte header validation `0x1F, 0x8B`, deflate header detection, corrupted header handling, short payload guard).

### Step 4: Unit Test Suite for Claude Message Turn Validator & Role Prefix Sanitizer
- **Target File**: `src-tauri/src/proxy/handlers/claude/tests_message_validator.rs`
- **Coverage**:
  - `message_turn_validator.rs` (validating alternating user/assistant turns, non-empty content assertions, system message normalization).
  - `role_prefix_sanitizer.rs` (stripping redundant role prefixes, trimming whitespace while preserving internal formatting).

### Step 5: Unit Test Suite for Auto-Switcher 5-Hour Quota Boundary Synchronizer & Reset Tracker
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_quota_and_cooldown.rs`
- **Coverage**:
  - `quota_boundary_synchronizer.rs` (synchronizing rolling 5-hour quota replenishment epochs, boundary alignment).
  - `quota_reset_epoch_tracker.rs` (historical reset tracking, dynamic quota calculation, expired boundary clearing).

### Step 6: Unit Test Suite for Consecutive Switch Cooldown Penalty & Velocity Throttle
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_consecutive_switch.rs`
- **Coverage**:
  - `consecutive_switch_cooldown.rs` (escalating cooldown penalties for rapid switching, cooldown expiration).
  - `consecutive_switch_throttle.rs` (switch velocity limits, multi-instance lockouts).

### Step 7: Unit Test Suite for Score Percentile Normalizer & Idle Candidate Score Decay
- **Target File**: `src-tauri/src/modules/auto_switcher/tests_score_and_decay.rs`
- **Coverage**:
  - `score_percentile_normalizer.rs` (mapping raw candidate scores to 0–1000 percentile range, handling single candidate edges).
  - `candidate_score_decay.rs` (exponential decay of stale candidate scores over time, baseline score preservation).

### Step 8: Unit Test Suite for Split-SQLite WAL Truncate & Passive Checkpoint Schedulers
- **Target File**: `src-tauri/src/modules/db_orm/tests_wal_schedulers.rs`
- **Coverage**:
  - `wal_truncate_scheduler.rs` (trigger conditions for WAL truncation, size threshold evaluation).
  - `wal_passive_checkpoint_scheduler.rs` (passive non-blocking checkpoint invocation, busy handler safety).

### Step 9: Unit Test Suite for Multi-Instance Environment Token Scrubber, PATH Normalizer & Port Guards
- **Target File**: `src-tauri/src/modules/instance/tests_sandboxed_environment.rs`
- **Coverage**:
  - `env_path_normalizer.rs` (PATH deduplication, GitMap priority injection, forbidden path removal).
  - `remote_port_guard.rs` (port lease reservation, collision detection in range 9222–9299, stale lease recovery).
  - Directory permission checks and security isolation.

### Step 10: Master End-to-End Test Plan and Pending Task Backlog Specification
- **Target Files**:
  - `.ai-memory/plans/pending/175-test-case-implementation-and-task-management.md`
  - `.ai-memory/plans/subtasks/175-test-case-implementation-and-task-management/subtask-1.md` through `subtask-5.md`
- **Coverage**:
  - Full catalog of deferred E2E tests ready for execution upon user prompt.
  - Comprehensive pending step audit (245 remaining steps across 5 tracks).

---

## 3. Strict Quality Invariants
- **Zero Build / Zero Test Execution**: Do not execute `cargo test`, `cargo build`, `npm test`, or `npm run build`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

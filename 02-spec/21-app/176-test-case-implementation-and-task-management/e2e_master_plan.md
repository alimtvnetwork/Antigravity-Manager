# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 176)

- **Slug**: `176-test-case-implementation-and-task-management`
- **Sequence**: 176
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 176. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-176-01: Instance Process Liveness & Affirmative Boolean Predicates
- **Category**: Multi-Instance Process Management
- **Pre-Conditions**: AGM daemon active with at least one registered instance in `instances.json`.
- **Stimulus**:
  1. Invoke `agm instances list --json` and capture reported process state.
  2. Spawn simulated child process and register PID in instance registry.
  3. Validate affirmative predicate evaluation (`is_instance_running_and_active`).
- **Expected Outcome**:
  - Predicate returns `true` exclusively when PID is nonzero, OS process matches binary signature, and heartbeat is valid.
  - Returns `false` without panicking when PID terminates.

### E2E-176-02: Transcript Early-Return Guard Scanning & Heartbeat Extraction
- **Category**: Running Prompt Discovery & Observer
- **Pre-Conditions**: Sandboxed workspace containing test transcript with `USER_INPUT`, `MODEL`, and empty trailing lines.
- **Stimulus**:
  1. Trigger prompt recovery scan on target workspace.
  2. Evaluate reversed line scan with guard clauses.
- **Expected Outcome**:
  - Skips empty, corrupt, or telemetry records via early return.
  - Successfully recovers genuine user prompt and media within 10 lines.

### E2E-176-03: Stale PID Reaper Daemon Auto-Reconciliation
- **Category**: Multi-Instance Lifecycle
- **Pre-Conditions**: Instance registry containing mock dead PIDs (`99999`) and live caller IDE PID.
- **Stimulus**:
  1. Trigger `StalePidReaper::reap_dead_instances`.
- **Expected Outcome**:
  - Reaps dead PID instances, transitions status to `Stopped`, and clears PID store.
  - Protects caller IDE PID and running instances from reaping.

### E2E-176-04: Profile State Hasher & Out-of-Band Configuration Drift Detection
- **Category**: Sandboxed Profile Isolation
- **Pre-Conditions**: Instance initialized with custom launch args and data paths.
- **Stimulus**:
  1. Generate baseline state hash via `ProfileStateHasher::compute_profile_hash`.
  2. Mutate custom argument or path out-of-band in `instances.json`.
  3. Recompute hash and trigger drift detection.
- **Expected Outcome**:
  - Detects hash mismatch immediately and flags configuration drift warning in audit log.

### E2E-176-05: IDE Binary Architecture Probe & Fallback Spawn Ladder
- **Category**: Process Spawning Safety
- **Pre-Conditions**: Valid binary and simulated missing/corrupt executable paths.
- **Stimulus**:
  1. Run `BinaryExecutionProbe::probe_binary` across missing path, zero-byte file, and host executable.
- **Expected Outcome**:
  - Missing and zero-byte files return structured error explaining invalidity without OS panic.
  - Host executable validates with `is_executable = true` and `error = None`.

### E2E-176-06: Inactive Account Staleness Detector & Auto-Probe Queueing
- **Category**: Auto-Switcher Routing
- **Pre-Conditions**: Multiple accounts with varying last-synced epochs (fresh, >30m, >60m).
- **Stimulus**:
  1. Execute candidate evaluation pass with `AccountStalenessDetector`.
- **Expected Outcome**:
  - Accounts older than 1800s flagged as `is_stale = true`.
  - Accounts older than 3600s prioritized with `requires_force_refresh = true`.
  - Clock-skewed future timestamps clamp to `elapsed = 0` via saturating subtraction.

### E2E-176-07: Sigmoid Headroom Score Multiplier & Zero Quota Gate
- **Category**: Candidate Scoring
- **Pre-Conditions**: Account candidate pool with quota ratios ranging from 0.0 to 1.0.
- **Stimulus**:
  1. Compute multipliers using `calculate_headroom_multiplier`.
- **Expected Outcome**:
  - Ample headroom (0.80) receives heavy bonus (> 0.85).
  - Depleted quota (0.0) yields exactly 0.0 multiplier, zeroing out candidate score.

### E2E-176-08: Split-SQLite Freelist Inspection & Incremental Vacuum
- **Category**: Split-SQLite Architecture
- **Pre-Conditions**: Physical SQLite database configured with `PRAGMA auto_vacuum = INCREMENTAL`.
- **Stimulus**:
  1. Insert and delete large rows to populate freelist pages.
  2. Call `PageFreelistAnalyzer::inspect_freelist`.
  3. Execute `execute_incremental_vacuum`.
- **Expected Outcome**:
  - Inspect correctly identifies free page count and calculates reclaimable bytes.
  - Incremental vacuum reclaims pages and reduces freelist count to 0.

### E2E-176-09: Connection Busy Timeout Dynamic Scaler Under Concurrent Load
- **Category**: Database Concurrency Tuning
- **Pre-Conditions**: Multi-connection SQLite pool with concurrent read/write threads.
- **Stimulus**:
  1. Simulate concurrency levels from 1 to 20 connections.
  2. Apply scaled timeout via `ConnectionBusyTimeoutScaler::apply_scaled_timeout`.
- **Expected Outcome**:
  - Single connection applies 5000ms.
  - Concurrency 5 scales to 10000ms.
  - High contention (>10) clamps cleanly at 15000ms.
  - Connection PRAGMA verifies live timeout.

### E2E-176-10: End-to-End Turn Sequence & Backlog Verification
- **Category**: Orchestration & Backlog Invariant
- **Pre-Conditions**: Completion of Steps 1 through 9.
- **Stimulus**:
  1. Verify zero build artifacts generated (`target/` untouched).
  2. Verify all subtasks marked completed in `.ai-memory/plans/subtasks/176-test-case-implementation-and-task-management/`.
  3. Verify parent plan moved to `.ai-memory/plans/completed/`.
- **Expected Outcome**:
  - Repository remains completely clean.
  - Backlog verified at 235 remaining steps across Tracks A through E.

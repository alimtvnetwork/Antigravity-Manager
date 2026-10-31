# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 177)

- **Slug**: `177-test-case-implementation-and-task-management`
- **Sequence**: 177
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 177. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-177-01: Auto-Switcher Candidate Affirmative Predicate & Quota Gate
- **Category**: Auto-Switcher Routing
- **Pre-Conditions**: Account candidate pool containing active, disabled, proxy-disabled, and quota-less accounts.
- **Stimulus**:
  1. Evaluate accounts using `is_account_active_and_enabled` and `is_candidate_eligible_for_switch`.
  2. Verify candidate filtering in `calculate_weekly_base_score` and `score_candidate_account`.
- **Expected Outcome**:
  - Disabled or proxy-disabled accounts immediately evaluate to `false` and yield `0.0` score without evaluating complex quota logic.
  - Accounts lacking quota structure evaluate to `false` under `is_candidate_eligible_for_switch`.

### E2E-177-02: Claude SSE Stream Empty Response Guard & Idle Timeout Recovery
- **Category**: Reverse Proxy Protocol Adapters
- **Pre-Conditions**: Mock Claude upstream returning empty initial chunk or triggering 120s idle gap.
- **Stimulus**:
  1. Dispatch streaming request through Claude proxy pipeline.
  2. Verify guard clause interception when `first_data_chunk` is `None`.
  3. Verify idle timeout emission of terminal `[DONE]` event.
- **Expected Outcome**:
  - Immediately triggers `StreamOutcome::Retry` upon `None` initial chunk without entering streaming loop.
  - Idle timeout terminates hanging streams safely after 120s.

### E2E-177-03: Multi-Instance Mock Keyring Writer & Bypass Token Verification
- **Category**: Multi-Instance Profile Sandboxing
- **Pre-Conditions**: Sandboxed instance data directory.
- **Stimulus**:
  1. Trigger mock credentials file generation via `KeyringMockWriter`.
  2. Validate OS keyring bypass flag persistence in `storage.json`.
- **Expected Outcome**:
  - Synthesizes mock tokens without accessing host OS keyring daemon.
  - Permissions and file integrity verify correctly.

### E2E-177-04: Multi-Instance Volatile Session File Reclaimer & Lock Sweep
- **Category**: Multi-Instance Process Hygiene
- **Pre-Conditions**: Simulated instance folder containing dead PID subdirectories and orphaned `.lock` files.
- **Stimulus**:
  1. Execute `SessionFileReclaimer::reclaim_session_files`.
- **Expected Outcome**:
  - Reclaims stale socket files and removes orphaned lockfiles.
  - Active caller instance files and live PIDs remain untouched.

### E2E-177-05: Multi-Instance CDP Port Conflict Detector & Eviction Range
- **Category**: Process Lifecycle & Port Management
- **Pre-Conditions**: Local ports bound in the default range `9222..9322`.
- **Stimulus**:
  1. Call `PortCollisionEvictor::find_or_evict_free_port`.
- **Expected Outcome**:
  - Accurately detects bound TCP ports.
  - Shifts allocation to the next free port in sequence without collisions.

### E2E-177-06: Reverse Proxy SSE Frame Delimiter Normalization & Fragment Concatenation
- **Category**: Reverse Proxy SSE Streaming
- **Pre-Conditions**: Upstream SSE emitter sending mixed delimiters (`\r\n\r\n`, single `\n`, unclosed chunks).
- **Stimulus**:
  1. Pipe raw byte stream through `SseDelimiterNormalizer`.
- **Expected Outcome**:
  - Frames are strictly normalized to canonical `\n\n` delimiters.
  - Partial chunks are buffered until boundary completion without byte loss.

### E2E-177-07: Reverse Proxy Client Request Cancellation Watcher & Abort Guard
- **Category**: Reverse Proxy Resilience
- **Pre-Conditions**: Upstream long-running response stream in progress.
- **Stimulus**:
  1. Simulate client TCP reset / stream drop.
  2. Inspect `RequestCancellationWatcher::is_cancelled`.
- **Expected Outcome**:
  - Cancellation watcher flags abort token immediately.
  - Halts upstream streaming task to prevent socket and quota leak.

### E2E-177-08: Auto-Switcher Candidate Quota Recovery Forecaster & Horizon Bounds
- **Category**: Quota & Candidate Scoring
- **Pre-Conditions**: Accounts with known quota reset epochs and consumption velocity.
- **Stimulus**:
  1. Query `QuotaRecoveryForecaster::forecast_recovery`.
- **Expected Outcome**:
  - Computes remaining hours until quota replenishment.
  - Clamps output cleanly between `0.0` and `168.0` hours (weekly window).

### E2E-177-09: Split-SQLite WAL Page Fragmentation Surveyor & Checkpoint Recommender
- **Category**: Split-SQLite Architecture
- **Pre-Conditions**: Active SQLite database with active WAL file.
- **Stimulus**:
  1. Inspect WAL file via `WalFragmentationSurveyor::survey_wal`.
- **Expected Outcome**:
  - Quantifies total WAL size, page count, and fragmentation index.
  - Recommends `Passive` checkpoint for small WALs and `Truncate` checkpoint when page fragmentation exceeds threshold.

### E2E-177-10: End-to-End Orchestration & Backlog Step Inventory Recalculation
- **Category**: Orchestration & Backlog Invariant
- **Pre-Conditions**: Successful completion of Steps 1 through 9.
- **Stimulus**:
  1. Verify zero build artifacts generated (`target/` untouched).
  2. Verify all subtasks marked completed in `.ai-memory/plans/subtasks/177-test-case-implementation-and-task-management/`.
  3. Verify parent plan moved to `.ai-memory/plans/completed/`.
- **Expected Outcome**:
  - Repository remains completely clean.
  - Backlog verified at 225 remaining steps across Tracks A through E.

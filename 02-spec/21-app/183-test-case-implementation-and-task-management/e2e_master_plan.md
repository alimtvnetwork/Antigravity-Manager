# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 183)

- **Slug**: `183-test-case-implementation-and-task-management`
- **Sequence**: 183
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 183. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-183-01: Claude Stop Sequence Exact and Partial Pattern Matching
- **Category**: Protocol Adapter IR Normalization
- **Pre-Conditions**: Stream buffer containing partial and exact Claude stop sequence markers (`"\\n\\nHuman:"`, `"\n\nAssistant:"`, `"STOP"`).
- **Stimulus**:
  1. Stream text tokens into `StopSequenceMatcher`.
  2. Evaluate `is_stop_sequence_matched` and `is_partial_match`.
- **Expected Outcome**:
  - Buffer holding partial suffix returns `is_partial_match == true` to withhold emission.
  - Complete stop sequence stops streaming and extracts clean text preceding the delimiter.

### E2E-183-02: Claude Stop Reason IR Mapping and Normalization
- **Category**: Protocol Adapter IR Normalization
- **Pre-Conditions**: Upstream Claude response payloads with varied `stop_reason` values (`end_turn`, `max_tokens`, `stop_sequence`, `tool_use`).
- **Stimulus**:
  1. Invoke `StopReasonNormalizer::to_gemini_finish_reason` and `from_gemini_finish_reason`.
  2. Verify affirmative predicate queries (`is_natural_stop`, `is_length_stop`, `is_tool_stop`).
- **Expected Outcome**:
  - `end_turn` maps to `STOP` (Gemini protocol).
  - `max_tokens` maps to `MAX_TOKENS`.
  - `tool_use` maps to `TOOL_CALLS`.
  - Unknown reasons gracefully default to `STOP`.

### E2E-183-03: Multi-Instance Profile Directory Validation & Hierarchy Containment
- **Category**: Multi-Instance Sandboxing
- **Pre-Conditions**: Temporary instance root containing valid, non-existent, and path-traversal attempted paths (`"../../etc"`).
- **Stimulus**:
  1. Validate candidate profile paths using `ProfileDirValidator::validate_profile_dir`.
- **Expected Outcome**:
  - Paths resolving outside the sandbox root are rejected with containment violations.
  - Valid nested profile directory paths are confirmed as safe.
  - Missing paths return non-existent directory warnings without panicking.

### E2E-183-04: Multi-Instance Workspace Storage Directory Sandboxing
- **Category**: Multi-Instance Sandboxing
- **Pre-Conditions**: Sub-instance `workspaceStorage` paths with SQLite databases, state files, and temporary caches.
- **Stimulus**:
  1. Execute `WorkspaceDirValidator::validate_workspace_storage`.
- **Expected Outcome**:
  - Confirms mandatory existence of valid workspace storage structure.
  - Guards against permission errors and read-only locks.

### E2E-183-05: Sandboxed Ephemeral Storage Cleanup & Symlink Safety
- **Category**: Multi-Instance Sandboxing
- **Pre-Conditions**: Isolated scratch workspace containing temp files, log dumps, and simulated junction/symlink pointers.
- **Stimulus**:
  1. Invoke `SandboxedStorageCleaner::clean_ephemeral_storage`.
- **Expected Outcome**:
  - Ephemeral files and cache folders are purged completely.
  - Symlinks and junctions are never traversed or unlinked outwards into host directories.

### E2E-183-06: Upstream SSE Streaming Chunk Inactivity & Idle Sentinel
- **Category**: Upstream Communication & Resilience
- **Pre-Conditions**: In-flight SSE upstream connection experiencing network stalls.
- **Stimulus**:
  1. Initialize `StreamingTimeoutSentinel` with 15ms inactivity threshold.
  2. Record chunks and introduce pauses longer than threshold.
- **Expected Outcome**:
  - Active chunks refresh the inactivity window.
  - Stalls beyond the threshold return `InactivityTimeout` error.
  - Exceeding total stream timeout trips `TotalDurationTimeout`.

### E2E-183-07: Upstream Dynamic Route Latency Weight EMA Smoother
- **Category**: Upstream Route Balancing
- **Pre-Conditions**: Route latency calculator tracking primary and fallback upstream endpoints.
- **Stimulus**:
  1. Feed latency observations of 100ms, 250ms, and 800ms into `RouteLatencyCalculator`.
  2. Query computed weights for routing selection.
- **Expected Outcome**:
  - EMA latencies smooth out single outlier spikes.
  - Route weights scale inversely with latency, clamped strictly between [0.1, 2.0].

### E2E-183-08: Auto-Switcher Consumption Acceleration & Runaway Burst Detector
- **Category**: Auto-Switcher Predictive Protection
- **Pre-Conditions**: Quota data series with token usage and timestamps.
- **Stimulus**:
  1. Pass normal linear consumption data points to `QuotaAccelerationDetector`.
  2. Pass runaway exponential consumption burst (> 1500 tokens/s²).
- **Expected Outcome**:
  - Linear consumption reports `Nominal` warning and `is_runaway_suspected == false`.
  - Burst consumption reports `Critical` warning and sets `is_runaway_suspected == true`.

### E2E-183-09: Split-SQLite Connection Pool Checkout Permitting & Concurrency Limiting
- **Category**: Split-SQLite ORM Concurrency
- **Pre-Conditions**: ConnectionCheckoutLimiter configured with max capacity of 4 background checkouts.
- **Stimulus**:
  1. Concurrently acquire checkout permits across multiple threads.
  2. Satiate permit pool and test acquisition rejection.
  3. Drop guards and test permit reclamation.
- **Expected Outcome**:
  - Exactly 4 permits granted concurrently; subsequent attempts return `None`.
  - Dropping permits restores capacity immediately and atomically without leaks.

### E2E-183-10: Master Multi-Instance Profile Launch & Sandboxed Execution Verification
- **Category**: Multi-Instance Full Lifecycle Integration
- **Pre-Conditions**: Mock profile directory with sandboxed workspace and port allocation.
- **Stimulus**:
  1. Run end-to-end sandbox preparation using `ProfileDirValidator` and `WorkspaceDirValidator`.
  2. Verify clean execution environment.
- **Expected Outcome**:
  - All multi-instance validation gates pass.
  - Ready for isolated launcher execution.

---

## 3. Backlog Inventory Tracking (Sequence 183 Post-Completion)

With Sequence 183 completed, the remaining steps across all tracks are:

- **Track A (Boolean Modernization & Guard Inversion)**: **0 steps remaining** (COMPLETED: Flattener, Message Turn Validator, Role Prefix Sanitizer, Stop Sequence Matcher, Stop Reason Normalizer)
- **Track B (Multi-Instance Sandboxing & Test Hardening)**: **1 step remaining** (Step 25: Master CLI Test Sandbox Orchestrator)
- **Track C (UI Accessibility, Contrast & Audit Modernization)**: **20 steps remaining**
- **Track D (Release Gates, Telemetry & Pipeline Guardrails)**: **15 steps remaining**
- **Track E (Continuous Improvements 569–720)**: **129 steps remaining** (Imp-569 to Imp-720)

**Total Remaining Backlog**: **165 steps** (decreased from 175).

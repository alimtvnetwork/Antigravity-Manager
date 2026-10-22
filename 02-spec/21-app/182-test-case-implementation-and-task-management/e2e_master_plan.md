# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 182)

- **Slug**: `182-test-case-implementation-and-task-management`
- **Sequence**: 182
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 182. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-182-01: Claude Turn Sequence Alternation & Whitespace Sanitization
- **Category**: Protocol Adapter IR Normalization
- **Pre-Conditions**: Raw Claude request containing empty user content strings and trailing whitespace.
- **Stimulus**:
  1. Validate payload using `MessageTurnValidator::validate_and_clean_messages`.
  2. Test turn alternation verification via `verify_role_alternation`.
- **Expected Outcome**:
  - Empty user content strings are backfilled with a minimal single period (`"."`).
  - Trailing newlines and spaces are stripped cleanly without altering message semantics.
  - Strict alternation (`user` -> `assistant` -> `user`) is validated, returning `false` on consecutive identical roles.

### E2E-182-02: Claude Consecutive Role Merging & Marker Stripping
- **Category**: Protocol Sanitization
- **Pre-Conditions**: Malformed Claude message array with consecutive user turns and embedded "Human:" prefixes.
- **Stimulus**:
  1. Strip leading prefixes using `strip_leading_role_markers`.
  2. Sanitize consecutive identical turns using `sanitize_claude_role_alternation`.
- **Expected Outcome**:
  - Consecutive user turns merge text with double newlines.
  - Non-text consecutive turns receive synthetic bridging turns (`"..."`).
  - Legacy conversational prefixes are stripped cleanly.

### E2E-182-03: Multi-Instance Child Process Escalating Kill Ladder
- **Category**: Process Lifecycle & Sandboxing
- **Pre-Conditions**: Running child process PID and simulated stubborn process.
- **Stimulus**:
  1. Invoke `ChildTerminationGuard::terminate_with_ladder`.
- **Expected Outcome**:
  - PID 0 and non-existent PIDs safely return `false` without panicking.
  - Active child processes are terminated within the bounded grace period.
  - Handles are closed cleanly without leaks.

### E2E-182-04: Multi-Instance Process Vitality Probe Direct Kernel Check
- **Category**: Process Vitality
- **Pre-Conditions**: Valid running process PID, dead PID, and PID 0.
- **Stimulus**:
  1. Query vitality via `ProcessVitalityProbe::is_pid_alive`.
- **Expected Outcome**:
  - Direct kernel check confirms vitality accurately without heavy `sysinfo` scans.
  - PID 0 and terminated PIDs immediately evaluate to `false`.
  - Current process evaluates to `true`.

### E2E-182-05: Multi-Instance IPC Socket Cleaner Stale File Sweeping
- **Category**: Socket and Lockfile Recovery
- **Pre-Conditions**: Temporary instance folder containing active vs abandoned IPC sockets (`vscode-ipc-*.sock`).
- **Stimulus**:
  1. Run `clean_stale_ipc_sockets` on directory.
- **Expected Outcome**:
  - Matching socket files are deleted cleanly.
  - Non-matching regular files (settings, databases) are strictly preserved.
  - Missing directories return zero errors.

### E2E-182-06: Reverse Proxy Upstream Endpoint Health Decay Penalty Recovery
- **Category**: Upstream Network Resilience (Imp-561)
- **Pre-Conditions**: `EndpointHealthTracker` with 60-second half-life configuration.
- **Stimulus**:
  1. Record connection failures via `record_failure`.
  2. Record subsequent successes via `record_success`.
- **Expected Outcome**:
  - Failure penalty decays exponentially over elapsed time.
  - Successes immediately accelerate penalty deduction (-0.5).
  - Health score computes accurately between 0.0 and 1.0.

### E2E-182-07: Upstream Endpoint Dynamic Weight Matrix Routing
- **Category**: Upstream Load Balancing (Imp-562)
- **Pre-Conditions**: Multiple registered upstream providers in `EndpointWeightMatrix`.
- **Stimulus**:
  1. Register endpoints with base weights (10.0 and 8.0).
  2. Degrade primary endpoint with `record_failure`.
  3. Query `best_endpoint`.
- **Expected Outcome**:
  - Selection smoothly transitions to secondary healthy endpoint upon primary failure.
  - Successes restore weight back toward base weight.
  - Ties break deterministically.

### E2E-182-08: Auto-Switcher Model-Aware Dynamic Quota Safety Floor
- **Category**: Auto-Switcher Anti-Abuse (Imp-563)
- **Pre-Conditions**: Account evaluation with varying remaining quota percentages (4.0%, 10.0%, 20.0%).
- **Stimulus**:
  1. Evaluate model tiers via `DynamicQuotaFloorGuard::evaluate` across lightweight, standard, heavyweight, and reasoning models.
- **Expected Outcome**:
  - Reasoning models enforce 12.0% floor; accounts below 12% evaluate as insufficient.
  - Lightweight models clear floor at >= 3.0%.
  - Headroom margin is calculated accurately.

### E2E-182-09: Split-SQLite Adaptive Page Cache Budgeting
- **Category**: Split-SQLite Resource Tuning (Imp-564)
- **Pre-Conditions**: System with 8GB available RAM and 8 active split-DB instances.
- **Stimulus**:
  1. Calculate cache budget via `calculate_cache_size_budget(8000, 8)`.
- **Expected Outcome**:
  - Target budget computes to 20MB (`-20480 KiB`).
  - Extreme RAM values clamp safely between `-2000 KiB` and `-64000 KiB`.
  - Negative PRAGMA cache_size convention is preserved.

### E2E-182-10: End-to-End Multi-Instance & Reverse Proxy Integration Pipeline Verification
- **Category**: Full Pipeline Integration
- **Pre-Conditions**: Isolated multi-instance sandbox environment with active SQLite stores and mock upstream proxy.
- **Stimulus**:
  1. Launch child process and verify vitality.
  2. Route streaming Claude calls through turn validator and role sanitizer.
  3. Clean up stale IPC sockets upon teardown.
- **Expected Outcome**:
  - Zero socket leaks, zero orphaned processes, and zero turn validation errors.
  - Complete round-trip pipeline integrity verified.

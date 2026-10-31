# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 178)

- **Slug**: `178-test-case-implementation-and-task-management`
- **Sequence**: 178
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 178. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-178-01: Telegram Inbound Node Identifier Validation & Machine Resolution
- **Category**: Inbound Remote Control Daemon
- **Pre-Conditions**: Telegram daemon initialized with mock command text payload.
- **Stimulus**:
  1. Feed command strings prefixed with `W1:`, `node2:`, `192.168.1.5:`, and invalid whitespace prefixes.
  2. Evaluate affirmative predicate `is_valid_node_identifier`.
  3. Validate `extract_node_targeted_command` tuple resolution.
- **Expected Outcome**:
  - Valid prefixes resolve cleanly to target node and payload without entering error branches.
  - Whitespace-tainted or malformed candidate strings safely evaluate to `None`.

### E2E-178-02: Email Inbound Heavy Action Rate-Limiting & Guard Inversion
- **Category**: Email Inbound Security & Dispatch
- **Pre-Conditions**: Mock authorized email message targeting `AccountRotate` and `SystemClean`.
- **Stimulus**:
  1. Dispatch heavy action twice within a 600-second window.
  2. Verify guard clause interception via `is_heavy_action_type`.
- **Expected Outcome**:
  - The second request is rate-limited by SQLite audit guard without triggering background process execution.
  - Audit log records `rate_limited_10m` status and returns graceful descriptive string.

### E2E-178-03: Multi-Instance Cloned MachineGuid Reseeding & Telemetry Independence
- **Category**: Profile Sandboxing & Anti-Collision
- **Pre-Conditions**: Cloned instance directory containing baseline `storage.json`.
- **Stimulus**:
  1. Invoke `reseed_storage_json` on profile `storage.json`.
  2. Inspect all 5 target keys: `telemetry.machineId`, `telemetry.devDeviceId`, `telemetry.sqmId`, `telemetry.macMachineId`, `storage.machineGuid`.
- **Expected Outcome**:
  - All 5 keys re-seeded with compliant UUID v4 strings.
  - Zero collision with parent or sibling instance IDs.

### E2E-178-04: Multi-Instance Cloned Telemetry Salting & sqmId Integrity
- **Category**: Telemetry Privacy & Profile Isolation
- **Pre-Conditions**: Cloned profile instance ID string.
- **Stimulus**:
  1. Invoke `ClonedTelemetrySalt::generate_salted_tokens`.
- **Expected Outcome**:
  - Produces isolated tokens salted with profile ID and nanosecond entropy.
  - `sqm_id` adheres strictly to uppercase braced syntax `{...}`.

### E2E-178-05: Multi-Instance Child Process Environment Variable Credential Scrubbing
- **Category**: Security Sandboxing & Host Isolation
- **Pre-Conditions**: Child process environment vector populated with `GITHUB_TOKEN`, `OPENAI_API_KEY`, and benign system keys (`PATH`, `HOME`).
- **Stimulus**:
  1. Pipe environment vector through `EnvTokenScrubber::scrub_environment`.
- **Expected Outcome**:
  - All sensitive cloud and operator tokens stripped from vector.
  - Benign system variables completely preserved.
  - Accurately reports scrubbed token count.

### E2E-178-06: Reverse Proxy Upstream Connection Pool Health Probing & Latency Degradation
- **Category**: Upstream Proxy Resilience
- **Pre-Conditions**: Active upstream connection pool routing requests to target LLM providers.
- **Stimulus**:
  1. Simulate consecutive upstream connection failures and elevated latencies (>2500ms).
  2. Query `ConnectionHealthProbe::health_state`.
- **Expected Outcome**:
  - Transitions to `Degraded` upon 2 consecutive failures or high average latency.
  - Transitions to `Dead` upon 5 consecutive failures, excluding endpoint from candidate pool.
  - Instant recovery upon successful probe response.

### E2E-178-07: Reverse Proxy HTTP/2 Keepalive Ping Scheduling & Stale Connection Eviction
- **Category**: HTTP/2 Protocol & Stream Keepalive
- **Pre-Conditions**: Long-running streaming connection with thinking models.
- **Stimulus**:
  1. Advance mock time past 30-second interval.
  2. Evaluate `H2PingScheduler::should_send_ping`.
  3. Simulate missing PONG responses exceeding timeout (10s) and max unacknowledged (3).
- **Expected Outcome**:
  - Emits keepalive ping frames at regular 30s intervals.
  - Flags connection as stale upon unacknowledged ping threshold to prompt transparent reconnect.

### E2E-178-08: Auto-Switcher Rapid Quota Drain Circuit Breaker & Cooldown Recovery
- **Category**: Quota Protection & Circuit Breakers
- **Pre-Conditions**: Account active with 100% quota percentage.
- **Stimulus**:
  1. Inject rapid quota consumption drop of 30% within a 10s window.
  2. Check `DrainCircuitBreaker::state` and `is_eligible`.
  3. Advance time past 180-second cooldown and provide stabilized observation.
- **Expected Outcome**:
  - Circuit breaker trips immediately upon > 25% drop, suppressing candidate account.
  - Transitions to `HalfOpen` after cooldown, and automatically resets to `Closed` once usage stabilizes.

### E2E-178-09: Split-SQLite Connection Lock Contention Watchdog & Peak Hold Duration
- **Category**: Split-SQLite Architecture
- **Pre-Conditions**: SQLite connection pool with concurrent transactions.
- **Stimulus**:
  1. Acquire lock and advance time past 3000ms and 8000ms thresholds.
  2. Call `LockContentionWatchdog::evaluate_contention`.
- **Expected Outcome**:
  - Flags `Contended` at >= 3000ms and `CriticalTimeout` at >= 8000ms with offender ID.
  - Accurately captures peak hold duration upon lock release.

### E2E-178-10: End-to-End Turn Sequence Verification & Backlog Step Inventory Recalculation
- **Category**: Orchestration & Backlog Invariant
- **Pre-Conditions**: Successful completion of Steps 1 through 9.
- **Stimulus**:
  1. Verify zero build artifacts generated (`target/` untouched).
  2. Verify all subtasks marked completed in `.ai-memory/plans/subtasks/178-test-case-implementation-and-task-management/`.
  3. Verify parent plan moved to `.ai-memory/plans/completed/`.
- **Expected Outcome**:
  - Repository remains completely clean.
  - Backlog verified at 215 remaining steps across Tracks A through E.

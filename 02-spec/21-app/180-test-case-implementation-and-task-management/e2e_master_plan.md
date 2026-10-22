# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 180)

- **Slug**: `180-test-case-implementation-and-task-management`
- **Sequence**: 180
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 180. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-180-01: Proxy Request Header Allowlist & Hop-by-Hop Stripping Verification
- **Category**: Reverse Proxy Security
- **Pre-Conditions**: Axum HeaderMap containing standard AI headers, custom headers, cookies, and RFC 2616 hop-by-hop headers.
- **Stimulus**:
  1. Process request headers through `HeaderAllowlistFilter::filter_headers`.
  2. Verify affirmative predicate `is_forwardable_header`.
- **Expected Outcome**:
  - `content-type`, `authorization`, and `anthropic-version` are forwarded.
  - `cookie`, `set-cookie`, `connection`, `upgrade`, and `transfer-encoding` are stripped cleanly.
  - No client-identifying or proxy-leak headers pass upstream.

### E2E-180-02: Email Inbound Legacy Prefix Command Extraction & Guard Inversion
- **Category**: Email Inbound Command Parsing
- **Pre-Conditions**: Inbound email subjects with varied legacy command prefixes (`project:`, `prompt:`, `ff:`, `exec:`, `agm switch`).
- **Stimulus**:
  1. Parse command subjects using `parse_legacy_prefix_command` and extracted guard helpers.
- **Expected Outcome**:
  - Prompt injection prefixes (`ai-task:`, `prompt:`) parse into `InboundAction::PromptInjection`.
  - `ff: node-1` extracts target node and maps to `InboundAction::FastForward`.
  - `exec:` extracts target IP and maps to `InboundAction::CliExecution`.
  - Unrecognized commands fall through to `InboundAction::Ignored` without side effects.

### E2E-180-03: Multi-Instance Window State Persistence, Fallback, & Restoration
- **Category**: Multi-Instance UI Layout
- **Pre-Conditions**: Transient instance directory without prior window state.
- **Stimulus**:
  1. Read window state via `WindowStateRestorer::load_bounds` (expect default 1280x800).
  2. Save custom maximized bounds (1920x1080) via `save_bounds`.
  3. Re-read and assert round-trip equality.
  4. Inject corrupted JSON and verify graceful fallback to default bounds.
- **Expected Outcome**:
  - Default coordinates are reliably provided for fresh instances.
  - Custom user window positioning persists across launches.
  - Corrupted files never cause application panic or crash.

### E2E-180-04: Multi-Instance Staggered Coordinate Calculation & Modulo Wrapping
- **Category**: Desktop Window Management
- **Pre-Conditions**: Multiple instances launched in rapid succession (indices 0 to 16).
- **Stimulus**:
  1. Compute screen coordinates using `calculate_staggered_coordinates`.
- **Expected Outcome**:
  - Instances 0 to 7 stagger diagonally by 40px on both X and Y axes.
  - Instance 8 wraps back to base coordinates, avoiding off-screen placement.
  - Window dimensions remain strictly 1280x800 across all instances.

### E2E-180-05: Multi-Instance Process Vitality Probe & Alive PID Verification
- **Category**: Process Lifecycle & Sandboxing
- **Pre-Conditions**: Current process PID, dead PID, and PID 0.
- **Stimulus**:
  1. Probe single PIDs via `is_process_alive`.
  2. Probe candidate list via `verify_instance_pids`.
- **Expected Outcome**:
  - PID 0 immediately evaluates to `false`.
  - Current process evaluates to `true`.
  - Dead/non-existent PIDs evaluate to `false`.

### E2E-180-06: Reverse Proxy Upstream Transport Metrics Ingestion & TTFB Moving Average
- **Category**: Upstream Network Resilience (Imp-553)
- **Pre-Conditions**: Multiple upstream endpoints with varying network latency profiles.
- **Stimulus**:
  1. Record successive transport timing samples using `TransportMetricsCollector::record`.
  2. Query `average_ttfb` for recorded and unrecorded endpoints.
- **Expected Outcome**:
  - Exponential moving average weights new samples at 20% and historical average at 80%.
  - Unrecorded endpoints safely return `None`.
  - Distinct upstream hosts maintain independent moving averages.

### E2E-180-07: Dynamic Connection Pool Sizing & Active Concurrency Clamping
- **Category**: Upstream Connection Management (Imp-554)
- **Pre-Conditions**: `DynamicPoolSizer` initialized with default settings.
- **Stimulus**:
  1. Record starting requests (`on_request_started`) across scaling workloads (1, 10, 100).
  2. Record request completions (`on_request_finished`).
- **Expected Outcome**:
  - Pool size scales dynamically with active request count plus 5 headroom.
  - Clamping enforces lower bound `MIN_IDLE_CONNECTIONS=5` and upper bound `MAX_IDLE_CONNECTIONS=50`.
  - Excess completions saturate safely without integer underflow.

### E2E-180-08: Auto-Switcher Hourly Switch Entropy & Oscillation Storm Guard
- **Category**: Auto-Switcher Anti-Abuse (Imp-555)
- **Pre-Conditions**: `HourlyEntropyGuard` configured with 1-hour rolling window and 0.35 oscillation threshold.
- **Stimulus**:
  1. Simulate repetitive oscillation between 1 or 2 accounts (6 switches).
  2. Simulate uniform switching across 5 distinct accounts.
- **Expected Outcome**:
  - Repetitive single/dual-account loop exhibits near-zero entropy and triggers `is_oscillating=true`.
  - Uniform multi-account switching exhibits high entropy (>0.9) and permits further switches.
  - Switch sequences under 5 events never trigger false-positive oscillation alarms.

### E2E-180-09: Split-SQLite Page Cache Memory Adjuster & Negative Pragma Sizing
- **Category**: Split-SQLite Resource Tuning (Imp-556)
- **Pre-Conditions**: In-memory SQLite database connection.
- **Stimulus**:
  1. Apply cache sizes (100 KiB, 4096 KiB, 100_000 KiB) via `PageCacheAdjuster::set_cache_size_kib`.
  2. Verify active cache setting via `get_current_cache_size`.
- **Expected Outcome**:
  - 100 KiB clamps up to 1024 KiB minimum (`-1024`).
  - 4096 KiB applies directly (`-4096`).
  - 100_000 KiB clamps down to 65536 KiB maximum (`-65536`).

### E2E-180-10: End-to-End Multi-Instance & Reverse Proxy Integration Pipeline Verification
- **Category**: Full Pipeline Integration
- **Pre-Conditions**: Sandbox environment with mock instance directory and local proxy listener.
- **Stimulus**:
  1. Configure sandboxed instance window bounds and staggered coordinates.
  2. Execute proxy request filtering and pool sizing concurrently.
- **Expected Outcome**:
  - Instance config and window states persist cleanly.
  - Request headers and pool limits adapt under concurrent load without contention.
  - Zero lockups or orphaned file remnants.

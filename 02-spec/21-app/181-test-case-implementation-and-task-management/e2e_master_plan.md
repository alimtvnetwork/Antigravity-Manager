# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 181)

- **Slug**: `181-test-case-implementation-and-task-management`
- **Sequence**: 181
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 181. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-181-01: Claude Content Block Flattener & Multimodal Extraction
- **Category**: Protocol Adapter IR Normalization
- **Pre-Conditions**: Heterogeneous Claude message payload containing text, image, tool_use, tool_result, and thinking blocks.
- **Stimulus**:
  1. Process payload through `flatten_claude_content`.
  2. Validate affirmative predicate checks (`is_valid_claude_role`, `is_text_block`, `is_tool_block`, `is_thinking_block`).
- **Expected Outcome**:
  - Text blocks and tool result outputs are concatenated with newline delimiters.
  - Image blocks, tool use blocks, and thinking blocks are accurately counted.
  - Normalized payload preserves total block count without memory leaks.

### E2E-181-02: Auto-Switcher Gemini Weekly Bucket Qualification & Loop Guard
- **Category**: Candidate Quota Scoring
- **Pre-Conditions**: Account with mixed quota buckets (Gemini weekly, 3p Claude buckets, 4h immediate windows).
- **Stimulus**:
  1. Evaluate buckets using `is_gemini_weekly_bucket`.
  2. Execute candidate scoring via `calculate_weekly_base_score`.
- **Expected Outcome**:
  - Non-Gemini or third-party buckets are bypassed immediately via early continue.
  - Valid weekly buckets compute remaining headroom and scale score proportionally.
  - Sub-8% quota buckets evaluate to 0 effective percentage.

### E2E-181-03: Multi-Instance Profile Structural Fingerprint Drift Detection
- **Category**: Multi-Instance Profile Integrity
- **Pre-Conditions**: Sandboxed instance configuration folder with `User/settings.json` and `User/keybindings.json`.
- **Stimulus**:
  1. Compute initial profile fingerprint via `compute_profile_fingerprint`.
  2. Mutate settings file and recompute fingerprint.
- **Expected Outcome**:
  - Deterministic 64-character SHA-256 hash returned on unchanged state.
  - Content mutation results in immediate hash drift detection.
  - Missing target files are fingerprinted via `MISSING:` token without I/O failure.

### E2E-181-04: Multi-Instance Remote Debugging Port Discovery & Collision Evasion
- **Category**: Process Isolation & CDP Debugging
- **Pre-Conditions**: Local host with ports in range [9222..9322] partially occupied.
- **Stimulus**:
  1. Allocate free port via `RemoteDebuggingPortAllocator::allocate_free_port`.
  2. Test active port detection via `is_port_available`.
- **Expected Outcome**:
  - Allocated port is strictly within valid bounds and free from TCP collisions.
  - Actively bound ports are detected and bypassed.
  - Port is released immediately upon probe completion.

### E2E-181-05: Multi-Instance Session Trace UUID Parsing & Correlation
- **Category**: Telemetry & Process Tracking
- **Pre-Conditions**: Formatted session trace string (`agm-inst-<tag>-<timestamp>-<random>`).
- **Stimulus**:
  1. Generate trace token via `SessionTraceUuid::generate`.
  2. Parse generated token via `SessionTraceUuid::parse`.
- **Expected Outcome**:
  - Trace token contains valid timestamp epoch and random token.
  - Instance tag is extracted accurately.
  - Corrupted or invalid strings return `None`.

### E2E-181-06: Upstream Latency Exponential Smoothing & Spike Damping
- **Category**: Upstream Network Routing (Imp-557)
- **Pre-Conditions**: `LatencyExponentialSmoother` with default configuration (`alpha = 0.2`, `initial = 150.0`).
- **Stimulus**:
  1. Ingest sequence of latency measurements (100ms, 120ms, 800ms spike, 110ms).
  2. Query `smoothed_latency` after each sample.
- **Expected Outcome**:
  - First sample establishes initial baseline.
  - Transient 800ms spike is damped smoothly without erratic routing fluctuation.
  - Samples below 1ms clamp to 1.0ms floor.

### E2E-181-07: Upstream HTTP/2 Flow Control Window Dynamic Tuning
- **Category**: Upstream Protocol Resilience (Imp-558)
- **Pre-Conditions**: Upstream HTTP/2 client connections for small vs large responses.
- **Stimulus**:
  1. Tune profile for streaming response (>4MB payload) via `tune_http2_flow_control`.
  2. Tune profile for lightweight request (<128KB payload).
- **Expected Outcome**:
  - Large payload expands stream window to payload size and connection window to 4x.
  - Small payload maintains conservative 128KB stream window.
  - Concurrency limits clamp within safe boundaries [16 .. 512].

### E2E-181-08: Auto-Switcher Candidate Score Variance & Outlier Dampening
- **Category**: Auto-Switcher Anti-Abuse (Imp-559)
- **Pre-Conditions**: Candidate pool with multiple normal scores and an extreme anomalous outlier.
- **Stimulus**:
  1. Apply variance dampener via `dampen_score_variance`.
- **Expected Outcome**:
  - Pools with fewer than 3 items or zero variance remain unaffected.
  - Extreme outliers exceeding `MAX_ALLOWED_Z_SCORE=2.5` have their effective scores clamped.
  - Normal scores remain unchanged.

### E2E-181-09: Split-SQLite MMAP Memory-Mapped I/O Architecture Tuning
- **Category**: Split-SQLite Resource Tuning (Imp-560)
- **Pre-Conditions**: Active SQLite database connection.
- **Stimulus**:
  1. Apply recommended mmap size via `MmapTuner::apply_mmap_size`.
  2. Query configured size via `query_current_mmap_size`.
- **Expected Outcome**:
  - 64-bit systems configure 256 MiB (`268_435_456`).
  - Connection executes PRAGMA successfully without errors.
  - Non-negative mmap size verified.

### E2E-181-10: End-to-End Multi-Instance & Reverse Proxy Integration Pipeline Verification
- **Category**: Full Pipeline Integration
- **Pre-Conditions**: Isolated test environment with multi-instance sandboxes and active upstream client.
- **Stimulus**:
  1. Initialize instance with fingerprinting and CDP port allocation.
  2. Route streaming Claude requests through latency smoother and flow controller.
- **Expected Outcome**:
  - Zero port collisions across concurrent instances.
  - Claude messages flattened, validated, and streamed without buffer stalls.
  - Zero memory leaks or orphaned socket listeners.

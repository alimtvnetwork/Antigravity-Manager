# Subtask 03: Upstream Streaming Timeout Sentinel & Route Latency Tests

- **Task**: Test Case Implementation and Task Management (Sequence 183)
- **Slug**: `183-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 6 & 7)
- **Status**: [COMPLETED]

## Scope
1. **Step 6 (Imp-565)**: Author unit tests for streaming timeout sentinel in `src-tauri/src/proxy/upstream/tests_streaming_timeout_sentinel.rs`.
   - Verified default initialization, record_chunk tracking.
   - Tested inactivity timeout trigger and AlreadyAborted lock.
   - Tested chunk resets for inactivity timer and total stream duration timeout.
   - Registered module in `src-tauri/src/proxy/upstream/mod.rs`.
2. **Step 7 (Imp-566)**: Author unit tests for route latency calculator in `src-tauri/src/proxy/upstream/tests_route_latency_calculator.rs`.
   - Tested default baseline weight for unobserved endpoints.
   - Tested first sample EMA initialization.
   - Tested multi-sample EMA smoothing progression.
   - Tested weight ratio clamping to [0.1, 2.0].
   - Tested route isolation across distinct endpoints.
   - Registered module in `src-tauri/src/proxy/upstream/mod.rs`.

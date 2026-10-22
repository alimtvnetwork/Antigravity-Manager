# Subtask 03: Upstream Latency Smoother & HTTP/2 Flow Controller Tests

- **Task**: Test Case Implementation and Task Management (Sequence 181)
- **Slug**: `181-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 6 & 7)
- **Status**: [COMPLETED]

## Scope
1. **Step 6 (Imp-557)**: Author unit tests for latency exponential smoother in `src-tauri/src/proxy/upstream/tests_latency_exponential_smoother.rs`.
   - Test initial latency recording and initialization.
   - Test alpha smoothing factor weighting.
   - Test outlier latency damping.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
2. **Step 7 (Imp-558)**: Author unit tests for HTTP/2 flow controller in `src-tauri/src/proxy/upstream/tests_http2_flow_controller.rs`.
   - Test initial flow-control window size (65535 bytes).
   - Test window reduction on stream chunk transmission.
   - Test window replenishment on WINDOW_UPDATE frame receipt.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

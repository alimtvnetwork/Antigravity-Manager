# Subtask 03: Upstream Transport Metrics & Dynamic Pool Sizer Tests

- **Task**: Test Case Implementation and Task Management (Sequence 180)
- **Slug**: `180-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 6 & 7)
- **Status**: [COMPLETED]

## Scope
1. **Step 6 (Imp-553)**: Author unit tests for transport metrics collector in `src-tauri/src/proxy/upstream/tests_transport_metrics_collector.rs`.
   - Test transport latency ingestion and sample counts.
   - Test exponential moving average TTFB calculations (80/20 weighting).
   - Test missing endpoint query handling.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
2. **Step 7 (Imp-554)**: Author unit tests for dynamic connection pool sizer in `src-tauri/src/proxy/upstream/tests_dynamic_pool_sizer.rs`.
   - Test active in-flight request tracking.
   - Test dynamic pool size clamping between [MIN_IDLE=5 .. MAX_IDLE=50].
   - Test saturating subtraction on request completion.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

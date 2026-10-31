# Subtask 03: Upstream Endpoint Health Decay & Weight Matrix Tests

- **Task**: Test Case Implementation and Task Management (Sequence 182)
- **Slug**: `182-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 6 & 7)
- **Status**: [COMPLETED]

## Scope
1. **Step 6 (Imp-561)**: Author unit tests for endpoint health decay in `src-tauri/src/proxy/upstream/tests_endpoint_health_decay.rs`.
   - Test exponential score decay over elapsed time.
   - Test half-life configuration.
   - Test recovery score bounds [0.0 .. 1.0].
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
2. **Step 7 (Imp-562)**: Author unit tests for endpoint weight matrix in `src-tauri/src/proxy/upstream/tests_endpoint_weight_matrix.rs`.
   - Test composite weight computation (latency + health + quota).
   - Test highest-weight candidate selection.
   - Test zero-weight fallback handling.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

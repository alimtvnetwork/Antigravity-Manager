# Subtask 02: Multi-Instance Unit Test Suites

- **Task**: Test Case Implementation and Task Management (Sequence 182)
- **Slug**: `182-test-case-implementation-and-task-management`
- **Subtask**: 02 (Steps 3, 4 & 5)
- **Status**: [COMPLETED]

## Scope
1. **Step 3**: Author unit tests for child termination guard in `src-tauri/src/modules/instance/tests_child_termination_guard.rs`.
   - Test graceful process exit timeout tracking.
   - Test escalation ladder transitions.
   - Test process handle cleanup.
   - Register module in `src-tauri/src/modules/instance.rs`.
2. **Step 4**: Author unit tests for process vitality probe in `src-tauri/src/modules/instance/tests_process_vitality_probe.rs`.
   - Test TTL-cached vitality results.
   - Test alive vs dead PID detection.
   - Test query debounce behavior.
   - Register module in `src-tauri/src/modules/instance.rs`.
3. **Step 5**: Author unit tests for IPC socket cleaner in `src-tauri/src/modules/instance/tests_ipc_socket_cleaner.rs`.
   - Test stale socket discovery and deletion.
   - Test missing directory resilience.
   - Test permission error handling.
   - Register module in `src-tauri/src/modules/instance.rs`.

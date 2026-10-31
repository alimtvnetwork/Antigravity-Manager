# Subtask 02: Multi-Instance Unit Test Suites

- **Task**: Test Case Implementation and Task Management (Sequence 180)
- **Slug**: `180-test-case-implementation-and-task-management`
- **Subtask**: 02 (Steps 3, 4 & 5)
- **Status**: [COMPLETED]

## Scope
1. **Step 3**: Author unit tests for window state restorer in `src-tauri/src/modules/instance/tests_window_state_restorer.rs`.
   - Test default window bounds (1280x800).
   - Test JSON round-trip serialization/deserialization.
   - Test invalid/corrupt state fallback.
   - Register module in `src-tauri/src/modules/instance.rs`.
2. **Step 4**: Author unit tests for window layout anchor in `src-tauri/src/modules/instance/tests_window_layout_anchor.rs`.
   - Test staggered window offset coordinates (40px stagger).
   - Test modulo 8 instance wrapping.
   - Test custom base coordinates.
   - Register module in `src-tauri/src/modules/instance.rs`.
3. **Step 5**: Author unit tests for process vitality scanner in `src-tauri/src/modules/instance/tests_vitality_scanner.rs`.
   - Test PID 0 rejection.
   - Test current process vitality (`std::process::id()`).
   - Test batch PID vitality vector mappings.
   - Register module in `src-tauri/src/modules/instance.rs`.

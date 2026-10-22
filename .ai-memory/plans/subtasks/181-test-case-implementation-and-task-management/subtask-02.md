# Subtask 02: Multi-Instance Unit Test Suites

- **Task**: Test Case Implementation and Task Management (Sequence 181)
- **Slug**: `181-test-case-implementation-and-task-management`
- **Subtask**: 02 (Steps 3, 4 & 5)
- **Status**: [COMPLETED]

## Scope
1. **Step 3**: Author unit tests for profile fingerprint hasher in `src-tauri/src/modules/instance/tests_profile_fingerprint_hasher.rs`.
   - Test deterministic SHA-256 fingerprint generation.
   - Test canonical key ordering resilience.
   - Test drift detection across modified fields.
   - Register module in `src-tauri/src/modules/instance.rs`.
2. **Step 4**: Author unit tests for remote debugging port allocator in `src-tauri/src/modules/instance/tests_remote_debugging_port_allocator.rs`.
   - Test port allocation in default range [9222 .. 9322].
   - Test sequential port incrementing and busy port skipping.
   - Test wrap-around behavior.
   - Register module in `src-tauri/src/modules/instance.rs`.
3. **Step 5**: Author unit tests for session trace UUID in `src-tauri/src/modules/instance/tests_session_trace_uuid.rs`.
   - Test standard UUID v4 formatting (36 chars, hyphens at positions 8, 13, 18, 23).
   - Test entropy and unique generation across iterations.
   - Test instance binding formatting.
   - Register module in `src-tauri/src/modules/instance.rs`.

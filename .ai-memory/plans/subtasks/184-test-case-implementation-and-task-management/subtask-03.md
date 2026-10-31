# Subtask 03: Upstream Request Size Guard & HTTP/2 Window Recovery Tests

- **Task**: Test Case Implementation and Task Management (Sequence 184)
- **Slug**: `184-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 6 & 7)
- **Status**: [COMPLETED]

## Scope
1. **Step 6 (Track E - Imp-569)**: Author unit tests for request size guard in `src-tauri/src/proxy/handlers/common/tests_request_size_guard.rs`.
   - Tested payload acceptance within default text ceiling (10 MB).
   - Tested payload acceptance within multimodal ceiling (50 MB).
   - Tested rejection on oversized payloads (`RejectedTooLarge`).
   - Tested model name multimodal capability detection (`is_multimodal_model_name`).
   - Registered module in `src-tauri/src/proxy/handlers/common.rs`.
2. **Step 7 (Track E - Imp-570)**: Author unit tests for HTTP/2 window recovery monitor in `src-tauri/src/proxy/upstream/tests_h2_window_recovery.rs`.
   - Tested consecutive flow-control stall recording.
   - Tested threshold evaluation triggering HTTP/1.1 downgrade (`should_downgrade_to_http1`).
   - Tested stall counter reset on stream success.
   - Tested concurrent multi-threaded stall accumulation.
   - Registered module in `src-tauri/src/proxy/upstream/mod.rs`.

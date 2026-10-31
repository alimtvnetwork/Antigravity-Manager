# Subtask 03: Request Header Sanitizer & Content Length Validator Tests

- **Task**: Test Case Implementation and Task Management (Sequence 187)
- **Slug**: `187-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 5 & 6)
- **Status**: [COMPLETED]

## Scope
1. **Step 5 (Track E - Imp-583)**: Author unit tests for request header sanitizer in `src-tauri/src/proxy/handlers/common/tests_request_header_sanitizer.rs`.
   - Test stripping of hop-by-hop headers (`connection`, `upgrade`, `keep-alive`, etc.).
   - Test stripping of proxy-internal telemetry (`x-agm-*`) and client identity headers (`cookie`, `x-real-ip`).
   - Test retention of allowed payload headers (`authorization`, `content-type`, `x-custom`).
   - Register module in `src-tauri/src/proxy/handlers/common.rs`.
2. **Step 6 (Track E - Imp-584)**: Author unit tests for content length validator in `src-tauri/src/proxy/handlers/common/tests_request_content_length_validator.rs`.
   - Test valid content length parsing within allowable bounds.
   - Test rejection of payload lengths exceeding default ceiling (32MB).
   - Test handling of chunked/absent headers and malformed non-ASCII/negative values.
   - Register module in `src-tauri/src/proxy/handlers/common.rs`.

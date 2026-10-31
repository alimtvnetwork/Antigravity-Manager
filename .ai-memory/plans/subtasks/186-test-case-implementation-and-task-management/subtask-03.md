# Subtask 03: Upstream Dynamic DNS Resolver & Retry Decelerator Tests

- **Task**: Test Case Implementation and Task Management (Sequence 186)
- **Slug**: `186-test-case-implementation-and-task-management`
- **Subtask**: 03 (Steps 5 & 6)
- **Status**: [COMPLETED]

## Scope
1. **Step 5 (Track E - Imp-578)**: Author unit tests for dynamic DNS resolver in `src-tauri/src/proxy/upstream/tests_dynamic_dns_resolver.rs`.
   - Test address resolution and in-memory TTL caching.
   - Test cache sweep removing expired IP entries.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
2. **Step 6 (Track E - Imp-579)**: Author unit tests for retry decelerator in `src-tauri/src/proxy/upstream/tests_retry_decelerator.rs`.
   - Test exponential backoff calculation across retry attempts.
   - Test failure pressure penalty scaling with rolling failures.
   - Test max delay clamping and bounded jitter.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

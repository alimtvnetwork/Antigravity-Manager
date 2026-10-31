# Subtask 02: Multi-Instance Unit Test Suites

- **Task**: Test Case Implementation and Task Management (Sequence 183)
- **Slug**: `183-test-case-implementation-and-task-management`
- **Subtask**: 02 (Steps 3, 4 & 5)
- **Status**: [COMPLETED]

## Scope
1. **Step 3**: Author unit tests for profile dir validator in `src-tauri/src/modules/instance/tests_profile_dir_validator.rs`.
   - Test profile root directory existence check.
   - Test directory traversal mitigation (`..`).
   - Test permission validation.
   - Register module in `src-tauri/src/modules/instance.rs`.
2. **Step 4**: Author unit tests for workspace dir validator in `src-tauri/src/modules/instance/tests_workspace_dir_validator.rs`.
   - Test workspace folder path accessibility.
   - Test missing directory auto-creation.
   - Test symlink escape defense.
   - Register module in `src-tauri/src/modules/instance.rs`.
3. **Step 5**: Author unit tests for sandboxed storage cleaner in `src-tauri/src/modules/instance/tests_sandboxed_storage_cleaner.rs`.
   - Test purging volatile cache directories (Code Cache, GPUCache, cache.nss).
   - Test preservation of persistent configuration (`settings.json`, SQLite databases).
   - Test error-tolerant cleanup on locked files.
   - Register module in `src-tauri/src/modules/instance.rs`.

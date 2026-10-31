# Subtask 04: Auto-Switcher Entropy Guard & SQLite Page Cache Adjuster Tests

- **Task**: Test Case Implementation and Task Management (Sequence 180)
- **Slug**: `180-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 8 (Imp-555)**: Author unit tests for hourly entropy and oscillation guard in `src-tauri/src/modules/auto_switcher/tests_hourly_entropy_guard.rs`.
   - Test Shannon entropy calculation across account switch sequences.
   - Test zero switches and uniform distribution scenarios.
   - Test oscillation storm detection threshold (<0.35).
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 9 (Imp-556)**: Author unit tests for split-SQLite page cache adjuster in `src-tauri/src/modules/db_orm/tests_page_cache_adjuster.rs`.
   - Test negative PRAGMA cache_size syntax generation.
   - Test KiB clamping within [1024 .. 65536].
   - Test in-memory SQLite connection execution and cache verification.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

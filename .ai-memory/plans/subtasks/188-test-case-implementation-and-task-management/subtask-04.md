# Subtask 04: Candidate Dedup, Commit Latency Observer & DB Size Watchdog Tests

- **Task**: Test Case Implementation and Task Management (Sequence 188)
- **Slug**: `188-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 7, 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 7 (Track E - Imp-590)**: Author unit tests for auto-switcher candidate deduplicator in `src-tauri/src/modules/auto_switcher/tests_candidate_dedup.rs`.
   - Test deduplication across identical account IDs keeping highest composite score.
   - Test deduplication across identical emails under differing account IDs.
   - Test score sorting order (descending by composite score).
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 8 (Track E - Imp-591)**: Author unit tests for commit latency observer in `src-tauri/src/modules/db_orm/tests_commit_latency_observer.rs`.
   - Test recording of commit durations and metric aggregation (count, avg, max).
   - Test slow commit threshold counting (> 50ms).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
3. **Step 9 (Track E - Imp-592)**: Author unit tests for split-SQLite database size watchdog in `src-tauri/src/modules/db_orm/tests_db_size_watchdog.rs`.
   - Test inspection of main database and WAL file sizes.
   - Test size threshold evaluation (`is_over_threshold`).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

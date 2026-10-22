# Subtask 04: Weekly Headroom Ratio, Index Cardinality & Query Latency Histogram Tests

- **Task**: Test Case Implementation and Task Management (Sequence 187)
- **Slug**: `187-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 7, 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 7 (Track E - Imp-585)**: Author unit tests for weekly headroom ratio in `src-tauri/src/modules/auto_switcher/tests_weekly_headroom_ratio.rs`.
   - Test linear ideal burn slope calculation (100% / 168 hours).
   - Test ratio calculation under surplus, on-track, and deficit conditions.
   - Test clamping to multiplier bounds [0.5, 1.5].
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 8 (Track E - Imp-586)**: Author unit tests for index cardinality analyzer in `src-tauri/src/modules/db_orm/tests_index_cardinality_analyzer.rs`.
   - Test selectivity ratio calculation (`COUNT(DISTINCT col) / COUNT(*)`).
   - Test low-selectivity flag trigger when selectivity < 5% and rows > 100.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
3. **Step 9 (Track E - Imp-587)**: Author unit tests for query latency histogram in `src-tauri/src/modules/db_orm/tests_query_latency_histogram.rs`.
   - Test bucket categorization (<1ms, 1-5ms, 5-20ms, >20ms).
   - Test atomic snapshot capture and counter reset.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

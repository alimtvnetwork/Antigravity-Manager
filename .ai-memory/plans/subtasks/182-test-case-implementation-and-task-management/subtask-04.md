# Subtask 04: Auto-Switcher Dynamic Quota Floor & SQLite Page Budget Tests

- **Task**: Test Case Implementation and Task Management (Sequence 182)
- **Slug**: `182-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 8 (Imp-563)**: Author unit tests for dynamic quota floor in `src-tauri/src/modules/auto_switcher/tests_dynamic_quota_floor.rs`.
   - Test floor calculation under normal vs heavy load.
   - Test clamping to valid bounds [5.0% .. 25.0%].
   - Test model-specific floor multipliers.
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 9 (Imp-564)**: Author unit tests for split-SQLite page budget controller in `src-tauri/src/modules/db_orm/tests_page_budget_controller.rs`.
   - Test budget allocation across active database instances.
   - Test memory cap clamping.
   - Test connection eviction / downscaling triggers.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

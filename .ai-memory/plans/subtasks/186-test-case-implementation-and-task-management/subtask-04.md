# Subtask 04: Auto-Switcher Binding Affinity & Split-SQLite Schema Audits Tests

- **Task**: Test Case Implementation and Task Management (Sequence 186)
- **Slug**: `186-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 7, 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 7 (Track E - Imp-580)**: Author unit tests for account binding affinity in `src-tauri/src/modules/auto_switcher/tests_binding_affinity.rs`.
   - Test affinity bonus applied to active account on instance.
   - Test zero affinity bonus for distinct candidate accounts.
   - Test custom bonus score overrides.
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 8 (Track E - Imp-581)**: Author unit tests for column default validator in `src-tauri/src/modules/db_orm/tests_column_default_validator.rs`.
   - Test audit of table column metadata via `PRAGMA table_info`.
   - Test detection of non-null columns missing default values (excluding PKs).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
3. **Step 9 (Track E - Imp-582)**: Author unit tests for index coverage verifier in `src-tauri/src/modules/db_orm/tests_index_coverage_verifier.rs`.
   - Test PRAGMA index_list and index_info parsing.
   - Test leading column query coverage evaluation (`is_query_covered`).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

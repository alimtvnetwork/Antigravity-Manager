# Subtask 01: Steps 1 & 2 - Boolean Logic Modernization & Guard Clause Inversion

- **Parent Plan**: `.ai-memory/plans/completed/176-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/176-test-case-implementation-and-task-management/spec.md#step-1-boolean-logic-modernization-in-instance-lifecycle`
- **Target Files**:
  - `src-tauri/src/modules/instance/lifecycle.rs`
  - `src-tauri/src/modules/repo_db/discovery.rs`
- **Status**: [COMPLETED]

## Execution Summary
1. Step 1: Modernized boolean logic in `src-tauri/src/modules/instance/lifecycle.rs`, extracting affirmative `is_instance_running_and_active` and `is_stale_lock_removable` helpers to eliminate compound negative expressions.
2. Step 2: Inverted nested condition trees in `src-tauri/src/modules/repo_db/discovery.rs` into early-return guard clauses (`extract_user_prompt_and_media`), flattening transcript scanning loops.

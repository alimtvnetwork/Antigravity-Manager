# Subtask 05: Strict FIFO Prompt Restore Query Validator
Status: COMPLETED
Sequence: 203
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/fifo_prompt_restore_validator.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_validator.rs`
Objective: Enforce AGENTS.md strict First-In-First-Out invariant: Inspects all prompt restore and query builder expressions across `active_prompts`, `prompt_backups`, and `repo_prompts.db`, verifying strict `ORDER BY created_at ASC, id ASC` and intercepting any inverted LIFO queries (`ORDER BY updated_at DESC`).
Lines: 158 lines implementation + 154 lines unit tests.

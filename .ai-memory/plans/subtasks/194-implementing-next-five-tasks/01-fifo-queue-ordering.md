# Subtask 01: FIFO Prompt Restore & Queue Invariant Guard
Status: COMPLETED
Sequence: 194
Owner: Subagent-1
Target Module: `src-tauri/src/modules/release_gate/fifo_queue_ordering_guard.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_fifo_queue_ordering.rs`
Objective: Validate that prompt restore, dispatch, and queue processing queries enforce strict `ORDER BY created_at ASC, id ASC` and intercept inverted LIFO patterns.
Completed Details:
- Enforced `ORDER BY created_at ASC, id ASC` across SQLite queue queries on active_prompts, prompt_backups, repo_prompts.
- Intercepted forbidden LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`).
- Authored unit test suite covering canonical FIFO validation, table detection, and point lookup exemptions.

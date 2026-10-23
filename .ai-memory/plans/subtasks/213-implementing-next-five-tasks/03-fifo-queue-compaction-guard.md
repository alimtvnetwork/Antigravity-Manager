# Subtask 03: FIFO Queue Compaction Guard (`Imp-692`)
Sequence: 213
Step: 278
Status: IN_PROGRESS
Module: `src-tauri/src/modules/db_orm/fifo_queue_compaction_guard.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs`

## Description
Coordinates SQLite database queue compaction and tombstone cleanup (`active_prompts`, `prompt_backups`), ensuring that compaction passes NEVER re-order rows, strictly maintain monotonically increasing `created_at ASC, id ASC`, and safely purge acknowledged rows without creating sequence gaps or transaction deadlocks.

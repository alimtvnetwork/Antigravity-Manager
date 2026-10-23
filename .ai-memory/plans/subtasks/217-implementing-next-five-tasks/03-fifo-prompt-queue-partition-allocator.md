# Subtask 03: FIFO Prompt Queue Partition & Priority Allocator (`Imp-712`)
Sequence: 217
Step: 298
Status: IN_PROGRESS
Module: `src-tauri/src/modules/db_orm/fifo_prompt_queue_partition_allocator.rs`

## Description
Implement the FIFO Prompt Queue Partition & Priority Allocator adhering to specification `02-spec/21-app/217-implementing-next-five-tasks/spec.md`:
1. Enforce AGENTS.md FIFO Restore & Queue Invariant:
   "All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
2. Implement `QueuePriorityBand`, `QueuePartitionBucket`, `AllocatedPromptItem`, and `PartitionAllocationReport`.
3. Provide `build_partition_query_sql` and `verify_allocated_partition_monotonicity`.
4. Create companion unit tests in `src-tauri/src/modules/db_orm/tests_fifo_prompt_queue_partition_allocator.rs`.

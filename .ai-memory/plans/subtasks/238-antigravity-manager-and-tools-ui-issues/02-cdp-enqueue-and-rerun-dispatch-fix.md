# Subtask 02: CDP Enqueue and Rerun Dispatch Fix

- **Parent Plan**: `.ai-memory/plans/pending/238-antigravity-manager-and-tools-ui-issues.md`
- **Slug**: `antigravity-manager-and-tools-ui-issues - Task 02`
- **Status**: PENDING

## Scope
1. Fix `src-tauri/src/commands/instance.rs` (`enqueue_prompt`) and `src-tauri/src/modules/repo_db/queue.rs`:
   - When target instance is running, submit prompt text directly into Antigravity IDE input field via DevTools CDP with `cancel_running_first: false`.
   - Ensure the IDE immediately receives and queues the prompt in its native conversation turn buffer.
   - Record prompt status in SQLite `active_prompts` and ensure optimistic injection in PromptTreeView.
2. Verify rerun option dispatches to the enqueue pipeline without dropping or stalling.
3. Validate and document why 1 prompt shows enqueued in FIFO during prompt review (honest SQLite queued turn count).
4. Verify Telegram inbound prompt dispatch leverages the same immediate queueing mechanism.

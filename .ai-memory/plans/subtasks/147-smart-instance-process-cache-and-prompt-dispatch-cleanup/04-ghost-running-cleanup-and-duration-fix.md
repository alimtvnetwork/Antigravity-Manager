# Subtask 04: Ghost Running Cleanup & Duration Formatter Hygiene

## Objective
Fix duration calculation errors (`NaNm NaNs`) and eliminate false `RUNNING` indicators across idle project cards and conversation trees.

## Actions
1. In `src/components/instances/PromptTreeViewModal.tsx`:
   - Implement `formatDuration` with strict NaN, Infinity, and negative guards (`!Number.isFinite(secs) || isNaN(secs) || secs < 0 ? '0s' : ...`).
   - Implement safe timestamp parser supporting both numeric epoch strings and ISO dates (`Date.parse` with fallback to `Date.now()`).
2. In `src-tauri/src/modules/repo_db.rs`:
   - Tighten `is_conv_running`:
     - If the conversation's latest step in the transcript is marked `DONE`, `ERROR`, or has no running tool calls, do NOT mark it `running` purely because file mtime was touched within 240 seconds.
     - Prune stale `active_prompts` rows: if an `active_prompt` marked `running` has had no updates for > 5 minutes or its corresponding `agy` worker process has exited, transition status to `'completed'` or `'idle'`.

# Subtask 218-02: Account Auto-Switching & Prompt Restore Resiliency
Task ID: `Imp-716`
Sequence: 218
Step: 302
Status: COMPLETED

## Scope
- Focus on prompt restore reliability during auto-switch and quota exhaustion.
- Fix `stop_prompt_goal_workers_for_instance` in `src-tauri/src/modules/repo_db/goal_heartbeat.rs` so terminated/stopped workers never mark active prompts as fake `'completed'`; transition them to `'backed_up'`.
- Fix `inspect_conversation_transcript` in `src-tauri/src/modules/repo_db/transcript.rs` so turns ending in `"error"`, 429, `RESOURCE_EXHAUSTED`, or quota depletion are NOT classified as `is_terminal_done`. Flag them as needing restore.
- Expand discovery freshness window in `discovery.rs` from 60 seconds to 30 minutes (1800s).
- Transition all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending`, `queued`, `blocked`) to `backed_up` without arbitrary time windows in `sweep_incomplete_prompts_for_instance`.
- Re-attribute unrestored prompts to the target instance during switch.
- Return honest `RestoreOutcome` and enforce unscoped fallback when scoped restore yields 0 records.

## Files Touched
- `src-tauri/src/modules/repo_db/goal_heartbeat.rs`
- `src-tauri/src/modules/repo_db/transcript.rs`
- `src-tauri/src/modules/repo_db/discovery.rs`
- `src-tauri/src/modules/repo_db/backup.rs`
- `src-tauri/src/modules/auto_switcher/rotation.rs`
- `src-tauri/src/modules/instance/switch.rs`

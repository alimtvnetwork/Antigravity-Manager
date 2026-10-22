# Subtask 02 — Re-attribution parity + honest restore outcomes (RC-2, RC-3, RC-6)

Repo: Antigravity-Manager. Spec: 01-agm-prompt-restore-spec.md
Status: PENDING. Owner: Worker wave 1.

## Goal
Manual switches restore exactly what auto-rotation restores, and every restore report tells the truth.

## Owned files
- src-tauri/src/modules/instance/switch.rs
- src-tauri/src/modules/backup_prompts_db/restore.rs
- src-tauri/src/modules/auto_switcher/rotation.rs (notification counts only)
- src-tauri/src/modules/instance/launch.rs, src-tauri/src/modules/instance/spawn.rs (scoped-restore fallback only)

## Steps
1. switch.rs Stage 1: after the existing `active_prompts` re-attribution, add the `prompt_backups` unrestored-row re-attribution that rotation.rs already performs (same UPDATE shape: unrestored rows only, source → target instance).
2. backup_prompts_db/restore.rs: stop swallowing the dispatch pass in `record_ignored`. Return a result struct {restored, dispatched, failed} and propagate dispatch failures as data (counts + last error), not silence. Keep best-effort semantics for the caller, but the counts must be real.
3. switch.rs audit: write `restored_prompts_count` from the actual Stage-5 restore result (thread the result back), not from `backed_up_count` at switch time.
4. rotation.rs: pass the real restored count to `notify_post_switch_prompt_status` (it currently passes `backed_up_count` twice).
5. RC-6: in backup_prompts_db/restore.rs, before `save_or_requeue_prompt`, apply the live-status dedup used by `resend_queued_prompts_for_instance` (repo_db/backup.rs): skip/preserve rows whose live status is `running` or `completed`.
6. launch.rs / spawn.rs reinject paths: if scoped restore returns 0 restored but unrestored records exist under other instance ids for the same projects, fall back to the unscoped restore semantics for those records.

## Verification (deferred to subtask 08)
- Code re-read of all three restore entry points for count honesty.
- E2E scenarios 1-2 in spec 03.

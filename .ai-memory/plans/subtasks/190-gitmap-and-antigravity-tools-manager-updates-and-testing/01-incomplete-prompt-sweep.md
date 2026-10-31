# Subtask 01 — Incomplete-prompt sweep + backup durability (RC-1, RC-5)

Repo: Antigravity-Manager. Spec: 02-spec/21-app/190-gitmap-and-antigravity-tools-manager-updates-and-testing/01-agm-prompt-restore-spec.md
Status: PENDING. Owner: Worker wave 1.

## Goal
No prompt that failed to complete across a switch is ever invisible to restore.

## Owned files
- src-tauri/src/modules/repo_db/backup.rs
- src-tauri/src/modules/backup_prompts_db/backup.rs
- src-tauri/src/modules/instance/workspace.rs (sweep re-run hook only)

## Steps
1. In `backup_running_prompts` (repo_db/backup.rs), remove the 120s recency condition on the `dispatched` → `backed_up` transition for the switch path: transition ALL rows with status IN (running, in_flight, dispatched) that are not `completed`, regardless of age. Keep the 2h stale-retirement behavior for non-switch callers if it is shared — if shared, add a parameter or a dedicated sweep function rather than changing all callers blindly.
2. Add `sweep_incomplete_prompts_for_instance(instance_id)` in repo_db/backup.rs: selects non-completed, non-queued rows (running/in_flight/dispatched, any age) for the instance and transitions them to `backed_up` FIFO (created_at ASC, id ASC). Return the swept count.
3. Call the sweep in `push_backed_up_prompts_after_channel` (instance/workspace.rs) for the target instance after the channel wait, before dispatch.
4. In backup_prompts_db/backup.rs `enforce_max_backup_prompts_cap`: never delete rows with `is_restored=0`; apply the cap only to restored rows.
5. In `backup_active_running_prompts_for_instance`: drop the 2h `updated_at` freshness filter for `queued` rows (keep it for discovery candidates if load requires — justify in code comment).

## Verification (deferred to subtask 08 — no builds/tests now)
- Code re-read: transitions cover all non-completed statuses; FIFO order preserved.
- E2E scenario 2 in spec 03 must pass at the end phase.

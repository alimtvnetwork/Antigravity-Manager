# Subtask 05: Instance Audit Trail Visibility

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `05-instance-audit-trail-visibility`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Fix the instance audit trail so that clicking on an instance or its audit button displays the full chronological history of lifecycle actions (`START`, `STOP`, `SWITCH_ACCOUNT`, `KILL`, `CONFIG_CHANGE`).

## Scope of Changes
- Inspect `src-tauri/src/modules/task_history_db.rs` schema and audit recording methods.
- Ensure instance management operations explicitly record `instance_id` (and instance name/profile) in the split-db history records.
- Fix frontend query / IPC call to fetch history entries filtered by `instance_id`.
- Provide a rich visual audit list with timestamps, status tags, action summaries, and diff details.

## Verification
- Performing start/stop/switch operations produces visible entries in the instance audit drawer.
- Drawer handles empty state gracefully when truly no actions have occurred yet, with actionable explanation.

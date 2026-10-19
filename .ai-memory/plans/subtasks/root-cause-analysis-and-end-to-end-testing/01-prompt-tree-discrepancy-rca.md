# Subtask 01: Prompt Tree Timestamp and Running Turn Discrepancy RCA

## Intent
Diagnose and resolve the discrepancy where running prompt turns in the IDE were not reflected in the Prompt Tree View, and timestamps/durations were inaccurately computed.

## Remediation Details
1. Live transcript file modification checks (`is_recent_active && is_owning_inst_alive`) override stale SQLite `IDLE` status in `src-tauri/src/modules/repo_db/tree_collect.rs`.
2. Active turns render an emerald pulsing `RUNNING` badge.
3. Pending turns waiting to be dispatched render an amber clock `QUEUED` badge.
4. Auto-scrolling implemented in `TreePanel.tsx` to automatically scroll to the active or latest turn upon conversation selection.

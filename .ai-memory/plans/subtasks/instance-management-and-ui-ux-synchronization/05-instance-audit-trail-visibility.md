# Subtask 05: Instance Audit Trail Logging & UI Modal Visibility

## Objectives
1. Ensure all instance lifecycle operations (`START`, `STOP`, `RESTART`, `KILL`, `CREATE`, `CLONE`, `DELETE`, `SWITCH`) log events into `task_history_db` (`task_index.db` and split history shards) tagged with `instance_id`.
2. Ensure SQL queries in `src-tauri/src/modules/task_history_db/switch_history.rs` expand aliases for the default instance (`""`, `NULL`, `"default"`, `"antigravity"`, `"agy"`, `"ide"`).
3. Ensure IPC response structure returns all alias fields (`recent_switches`, `total_switches`, `records`, `items`).
4. Modernize `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`:
   - Dual-tab toggle: `Switch Cycles ({totalSwitches})` vs `All Operations ({totalOperations})`.
   - Automatic fallback: if `recentSwitches.length === 0` and `totalOperations > 0`, default to `operations` tab automatically.
   - Comprehensive badge styles for action codes 1–14.

## Owned Files
- `src-tauri/src/modules/task_history_db/switch_history.rs`
- `src-tauri/src/modules/audit_action.rs`
- `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`

## Verification
- Querying audit trail for an instance returns logged lifecycle events.
- Modal renders entries cleanly without blank fallback card when operations exist.

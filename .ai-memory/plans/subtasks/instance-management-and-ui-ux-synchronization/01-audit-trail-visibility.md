# Subtask 01: Audit Trail Visibility for Instances

Owner: Worker 01 / Lead
Target Files:
- `src-tauri/src/modules/task_history_db/switch_history.rs`
- `src-tauri/src/modules/account/switch.rs`
- `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`
- `src/services/instanceService/projects.ts`

## Requirements
1. In `src-tauri/src/modules/account/switch.rs`:
   Normalize `target_ide`: if `target_ide` is `"antigravity"`, `"agy"`, `"ide"`, `"classic"`, or `None`, normalize to `"default"` before passing to `AuditTask::start`.
2. In `src-tauri/src/modules/task_history_db/switch_history.rs`:
   - In `get_instance_switch_history`: expand `switch_cond` so that when `?1 == 'default'`, it matches `'default'`, `''`, `NULL`, `'antigravity'`, `'agy'`, `'ide'`, and `'classic'`.
   - In `get_instance_audit_trail`: similarly expand `cond` so that `'default'` matches all default representations.
3. In `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`:
   - Fetch both `getInstanceSwitchHistory` and `getInstanceAuditTrail`.
   - Provide toggle tabs: `Switch Cycles ({count})` and `All Events ({count})`.
   - If switch cycles are empty but general audit trail entries exist, automatically default to the `All Events` tab so the user sees real activity (prompt backups, restores, PID sync, dispatches) instead of a blank empty state.
   - For general audit trail records, display timestamp, action badge, status indicator, duration, and collapsible detail rows. 
22: ## Status: COMPLETED
23: - Implemented alias expansion in task history DB queries and hot task cache.
24: - Concurrently fetched switch and audit history in InstanceAuditTrailModal with segmented pill toggle.
25: - Auto-defaulted to All Operations tab when switch records are 0.

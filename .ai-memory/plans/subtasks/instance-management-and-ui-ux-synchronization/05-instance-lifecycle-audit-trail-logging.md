# Subtask 05: Instance Lifecycle Audit Trail Full-Stack Logging

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `IN PROGRESS`  
**Owner:** Worker 03 (Full-Stack / Backend)  
**Exclusive Files:**  
- `src-tauri/src/modules/audit_action.rs`  
- `src-tauri/src/modules/task_history_db/instance_lifecycle.rs`  
- `src-tauri/src/modules/instance/create.rs`  
- `src-tauri/src/modules/instance/launch.rs`  
- `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`  

## Objectives & Deliverables
1. Add `InstanceLifecycle = 6` to `AuditAction` enum or dedicated action codes for instance operations.
2. Implement `log_instance_event(instance_id, action, detail)` recording into `task_history_db`.
3. Call `log_instance_event` during:
   - `create_instance` / `copy_instance`
   - `launch_instance`
   - `stop_instance`
   - `restart_instance`
   - `switch_account_to_instance`
4. Update `InstanceAuditTrailModal.tsx` to display lifecycle operations seamlessly when `operations` tab is selected, with clear icons, action labels, and timestamps.

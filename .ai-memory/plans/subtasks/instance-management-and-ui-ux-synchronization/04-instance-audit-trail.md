# Subtask 04: Instance Audit Trail Telemetry & Display

## Scope
- Files: `src-tauri/src/modules/task_history_db.rs`, `src-tauri/src/commands/audit.rs`, `src/components/instances/InstanceAuditModal.tsx`

## Implementation Steps
1. In `task_history_db.rs` & `commands/audit.rs`:
   - Implement query command `get_instance_audit_trail(instance_id: String) -> Result<Vec<AuditRecord>, AppError>`.
   - Record account switches and start/stop events into audit tables.
2. In `InstanceAuditModal.tsx`:
   - Call `get_instance_audit_trail` on mount.
   - Render chronological event timeline with icons and details.
   - Render clean empty state if no history exists yet.

## Verification
- IPC command return check and UI rendering verification.

# Subtask 01: UI List View Alignment, PID Formatting & Switch Account Dialog Modernization

## Task Metadata
- **Code**: `Task-01`
- **Owner**: Worker 01
- **Status**: PENDING
- **Owned Files**:
  - `src/components/instances/table/InstanceRowCells.tsx`
  - `src/pages/instances/InstanceCard.tsx`
  - `src/pages/instances/SwitchAccountDialog.tsx`
  - `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`

## Requirements
1. **PID Clarification**:
   - In `InstanceRowCells.tsx` (`StatusCell`): Render `● Running (PID: ${inst.pid})` inside a status capsule (`bg-teal-500/10 text-teal-400 border border-teal-500/20` or emerald).
   - In `InstanceCard.tsx`: Render `Running (PID: ${inst.pid})`.
2. **List View Path Alignment**:
   - In `InstanceRowCells.tsx` (`IdeExeAndDataCell`): Render the trailing executable name and trailing data path **together on the same horizontal row** (`flex flex-row items-center gap-2`), with monospace styling and copy buttons.
3. **Switch Account Dialog Modernization**:
   - In `SwitchAccountDialog.tsx`:
     - Eliminate foreign `teal-*` styling; apply AGM dark-glass styling (`rounded-2xl`, glass backdrop, subtle borders, sky/amber badges).
     - Add `Escape` keydown handler to close dialog.
     - Wrap header action and footer into contiguous segmented capsules.
4. **Audit Trail Modal Fallback & Badge Styling**:
   - In `InstanceAuditTrailModal.tsx`:
     - Add badge styles for action codes 6–12 (Launch, Stop, Kill, Create, Clone, Delete, Error).
     - Default to "All Operations" if `recent_switches.length === 0` and operations exist.

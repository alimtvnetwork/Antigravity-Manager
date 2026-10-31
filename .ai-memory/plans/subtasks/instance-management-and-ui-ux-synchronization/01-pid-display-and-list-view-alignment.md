# Subtask 01: PID Display & Inline List View Layout

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `IN PROGRESS`  
**Owner:** Worker 01 (Frontend)  
**Exclusive Files:**  
- `src/components/instances/table/InstanceRowCells.tsx`  
- `src/components/instances/InstanceTable.tsx`  
- `src/pages/instances/InstanceCard.tsx`  

## Objectives & Deliverables
1. Confirm that the numeric status (e.g. `1632`) is the Process ID (`PID`).
2. Update `InstanceRowCells.tsx`:
   - Combine `Status & PID`, executable ending path, and data directory ending path onto the **exact same horizontal line**.
   - Use an emerald/green pill badge clearly stating `PID: <pid>` (or `Idle` when stopped).
   - Display `<Terminal>` icon + truncated `.exe` ending + separator `·` + `<Folder>` icon + truncated data dir ending.
3. Update `InstanceCard.tsx` to similarly format the status and paths cleanly without awkward vertical gaps.
4. Ensure no horizontal scroll blowout or broken flex wrapping.

# Subtask 02: Instances Table Layout, Column Combination & Prompt Button Alignment

Owner: Worker 02 / Frontend
Target Files:
- `src/components/instances/InstanceTable.tsx`
- `src/pages/instances/`

## Requirements
1. **Combine Executable & Data Directory Columns**:
   - Merge `IDE Executable` and `Data Directory` into a single high-density column `IDE EXE & DATA`.
   - Render the ending filename/folder (`Antigravity.exe` and `...\\default-copy-5597`) vertically stacked.
2. **Expose Prompt Button Outside**:
   - Move the `Prompts` button out of the `...` overflow menu.
   - Render it as a primary dark-glass action button directly alongside `Switch` and `Stop`.
3. **Table Layout Overflow Containment**:
   - Stop list view from running off the right-hand edge by enforcing `max-w-full overflow-x-auto` and bounded column widths.

## Status: COMPLETED
- Merged IDE Executable and Data Directory into unified `IDE EXE & DATA` column.
- Moved Prompts button outside into the primary action capsule.
- Enforced 5-column proportional table layout with overflow protection.

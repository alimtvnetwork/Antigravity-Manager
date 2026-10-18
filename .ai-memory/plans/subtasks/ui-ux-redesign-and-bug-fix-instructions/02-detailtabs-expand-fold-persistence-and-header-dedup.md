# Subtask 02: DetailTabs Expand/Fold Persistence & DetailHeader De-duplication

**File**: `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/02-detailtabs-expand-fold-persistence-and-header-dedup.md`  
**Parent Plan**: `.ai-memory/plans/ui-ux-redesign-and-bug-fix-instructions.md`  
**Status**: Pending  

## Objectives
1. In `src/components/instances/PromptTreeView/DetailTabs.tsx`:
   - Decouple `hasExpandToggle` from `previewTab === 'instruction'`. Ensure the `Expand Full` / `Collapse` toggle button remains visible and functional whether viewing `Input` or `Output`.
   - Maintain the expand/collapsed view state smoothly across tab switches.
2. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - De-duplicate repetitive titles and IDs. Remove duplicate conversation ID from the header row.
   - Retain a clean, prominent conversation title and status badge.
   - Encapsulate raw UUIDs (`ID: 81170dc5-...`) and internal sequence numbers into the Details modal.

# Subtask 05: Compact Button Layout, Reduced Rounding & Details Button

## Scope
- Files: `src/components/instances/PromptTreeView/DetailTabs.tsx`, `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/TreePanel.tsx`

## Implementation Steps
1. In `DetailTabs.tsx`:
   - Group `[ Input | Output | Expand Full ]` inside the right-hand segmented capsule.
2. In `DetailHeader.tsx`, `DetailTabs.tsx`, and `TreePanel.tsx`:
   - Replace standalone `rounded-full` buttons with clean `rounded-md`/`rounded-lg` segmented capsules with subtle dividers.
3. In `DetailHeader.tsx`:
   - Restore the `[ Details ]` button in the action bar, forwarding instance, IDE, PID, and turn metadata to `DetailsModal.tsx`.

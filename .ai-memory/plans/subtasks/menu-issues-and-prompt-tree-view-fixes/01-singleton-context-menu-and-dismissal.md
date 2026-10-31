# Subtask 01: Singleton Context Menu Architecture & Dismissal Guard

## Status: Completed (Verified)
## Assigned Files:
- `src/components/instances/PromptTreeView/TreePanel.tsx`
- `src/components/instances/PromptTreeView/PromptTreeContextMenu.tsx`

## Verification Outcome:
- Hoisted context menu state to `TreePanel.tsx` (`activeContextMenu`).
- Mounted transparent full-screen backdrop layer (`fixed inset-0 z-[99998] bg-transparent`) to capture outside clicks and unmount the menu immediately.
- Bound window capture listeners for `pointerdown`, `contextmenu`, `keydown` (`Escape`), `scroll`, and `blur` in `PromptTreeContextMenu.tsx`.
- Removed 100ms click dropping guard so clicks are never dropped.
- Verified TypeScript build clean (code 0).

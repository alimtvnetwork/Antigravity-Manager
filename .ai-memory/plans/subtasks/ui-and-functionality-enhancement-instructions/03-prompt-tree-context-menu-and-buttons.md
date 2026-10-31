# Subtask 03: Prompt Tree Context Menus & Button Rounding

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-03`  
**Status:** `READY`

## 1. Objectives & Scope
- Implement right-click context menu on tree nodes (`ProjectNode.tsx`, `ConversationNode.tsx`, `PromptTreeContextMenu.tsx`).
- Prevent premature context menu dismissal by scoping scroll listeners to `.tree-scroll-container` rather than window capture.
- Reduce excessive button rounding in `DetailHeader.tsx`, `DetailTabs.tsx`, `TreePanel.tsx` (replace `rounded-full` with crisp segmented capsules: `rounded-md` / `rounded-[4px]`).
- Group `[Input | Output | Expand Full]` into a contiguous right-aligned segmented bar.

## 2. Disjoint File Ownership
- `Owned Files:` `src/components/instances/PromptTreeView/PromptTreeContextMenu.tsx`, `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/DetailTabs.tsx`, `src/components/instances/PromptTreeView/ProjectNode.tsx`, `src/components/instances/PromptTreeView/ConversationNode.tsx`

## 3. Verification Evidence Contract
- Right-clicking tree item triggers context menu without premature dismissal.
- Buttons display crisp `rounded-md` corners and segmented layout.

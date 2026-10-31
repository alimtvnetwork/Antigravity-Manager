# Subtask 03: Prompt Tree View UI/UX Refactoring & Reduced Button Rounding

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-03`  
**Status:** `READY`

### Objective
Modernize Prompt Tree View layout, eliminate excessive rounding, and add context menus:
1. Replace `rounded-full` with `rounded-md` / `rounded-[5px]` across action buttons and view pills in `DetailHeader.tsx`, `DetailTabs.tsx`, and `TreePanel.tsx`.
2. Compact view mode buttons `[Input | Output | Expand Full]` into a contiguous segmented capsule anchored to the right-hand side.
3. Implement right-click context menu on tree sidebar items (Project, Conversation, Turn) with support for Copy, Rerun, Raw View, and Telemetry.
4. Elevate enqueued prompts into a permanent FIFO Queue panel directly below the running prompt view in `PromptInstructionView.tsx`.
5. Enhance active conversation selection styling for high visual contrast.

### Target Files
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/DetailTabs.tsx`
- `src/components/instances/PromptTreeView/TreePanel.tsx`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/ConversationNode.tsx`
- `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`

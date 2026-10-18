# Subtask 04: Tree Panel Empty State Resolution & Image Rendering

**File**: `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/04-prompt-tree-empty-state-and-image-rendering.md`  
**Parent Plan**: `.ai-memory/plans/ui-ux-redesign-and-bug-fix-instructions.md`  
**Status**: Pending  

## Objectives
1. In `src/components/instances/PromptTreeView/TreePanel.tsx` & `usePromptTreeState.ts`:
   - Diagnose why "No projects or conversations found." appeared even when 47 projects were loaded and a prompt was actively running.
   - Fix filtering logic so running and queued prompts are always prominently indexed, never swallowed by default filters or mismatched activeFilter states.
   - Ensure `Running` and `Queued` filter pills accurately partition and highlight active executions.
2. Multimodal Prompt Image Visibility:
   - Ensure images in prompts (whether base64, data URL, or filesystem path) render cleanly in `PromptInstructionView.tsx` and `markdown.tsx`.
   - Provide copy options: "Copy Image as Markdown", "Copy Image for IDE / OneNote".

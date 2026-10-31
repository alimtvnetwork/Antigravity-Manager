# Subtask 02: Prompt Tree View Copy Button Attachments

**Task Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Subtask**: `02-subtask-copy-button-attachments`  
**Sequence**: `243-02`  
**Spec Reference**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/02-component-and-attachment-spec.md`  

## Objectives
1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Refactor `handleCopyAll()` to resolve active media paths from selected turn and conversation.
   - Run prompt text through `formatPromptWithAttachments()` before writing to clipboard.
2. In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
   - Ensure `handleCopyAllWithRichImages` and prompt AI copies use canonical attachments.
3. Verify Send, Enqueue, and Rerun preserve canonical attachment blocks.

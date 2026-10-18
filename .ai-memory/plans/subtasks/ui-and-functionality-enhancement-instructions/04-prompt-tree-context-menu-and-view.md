# Subtask 04: Prompt Tree Right-Click Context Menu & Distinct Conversation View

## Scope
- Files: `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`, `src/components/instances/PromptTreeView/ProjectContextMenu.tsx`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`

## Implementation Steps
1. In `ConversationContextMenu.tsx` and `ProjectContextMenu.tsx`:
   - Ignore right-click (`e.button === 2`) in `handleClickOutside` to prevent premature portal unmounting.
   - Inject actions: View Details, Copy with Images, Copy without Images, Copy Image to Clipboard, Export Conversation.
2. In `PromptInstructionView.tsx`:
   - When `selectedTurnNumber === null`, render Conversation Session Dashboard with metrics, queued turns, and turn strip.
   - When `selectedTurnNumber !== null`, render Prompt Turn Inspector.

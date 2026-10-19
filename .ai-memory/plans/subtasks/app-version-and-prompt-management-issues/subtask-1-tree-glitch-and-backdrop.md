# Subtask 1: Fix Prompt Tree Selection Glitching & Modal Backdrop Dismissal

## Objective
1. Eliminate prompt text glitching/flickering in `PromptInstructionView.tsx` during background sync polling (`Sync: 30s`). Preserve `selectedConversationId` and `selectedTurnId` across data refreshes.
2. Enable click and double-click dismissal on the modal backdrop in `PromptTreeViewModal.tsx`.

## Key Files
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- `src/components/instances/PromptTreeView/usePromptTreeStore.ts`

## Verification
- Modal closes on backdrop click/double-click.
- Selected prompt and turn remain locked during 30s background sync cycles.

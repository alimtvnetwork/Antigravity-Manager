# Subtask 02: Enqueued Items Visibility & Auto-Expansion

## Status: Completed (Verified)
## Assigned Files:
- `src/components/instances/PromptTreeView/conversationGuards.ts`
- `src/components/instances/PromptTreeView/tierClassification.ts`
- `src/components/instances/PromptTreeView/ProjectNode.tsx`
- `src/components/instances/PromptTreeView/ConversationNode.tsx`

## Verification Outcome:
- Exported `isConversationOrTurnQueued(conv)` in `conversationGuards.ts`.
- Removed local context menu state and duplicate `<PromptTreeContextMenu>` JSX portals from `ConversationNode.tsx` and `ProjectNode.tsx`.
- Guaranteed auto-expansion in `ConversationNode.tsx` (`isExpanded = true`) when queued turns exist or when `activeFilter === 'queued'`.
- Preserved conversations with queued turns in `activeFilter === 'queued'` in `ProjectNode.tsx` and sorted them into Tier 2.
- Verified TypeScript build clean (code 0).

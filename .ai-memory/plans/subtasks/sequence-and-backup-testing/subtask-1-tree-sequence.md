# Subtask 1: Prompt Tree View 3-Tier Sequence Verification

## Objective
Verify that the Prompt Tree View sequences conversations and prompt turns according to the strict 3-tier ordering hierarchy:
1. Tier 1: `RUNNING` prompts on top.
2. Tier 2: `QUEUED` prompts in the middle with amber badges.
3. Tier 3: Completed/historical prompts sorted by timestamp descending.

## Files Involved
- `src/components/instances/PromptTreeView/conversationGuards.ts`
- `src/components/instances/PromptTreeView/ConversationNode.tsx`
- `src/components/instances/PromptTreeView/ProjectNode.tsx`
- `src/components/instances/PromptTreeView/tierClassification.ts`
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`

## Verification Strategy
- Create automated test verifying `sortConversations` and `getConversationStatus` contracts.
- Validate that queued turns cause their parent conversation to be classified as Tier 2 and auto-expand.
- Confirm that running prompts are never demoted beneath queued or historical prompts.

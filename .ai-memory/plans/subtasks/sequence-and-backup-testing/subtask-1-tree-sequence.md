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

## Status: Completed (Verified)
- **Unit Test File**: `src/components/instances/PromptTreeView/__tests__/treeSequence.test.ts`
- **Results**: 100% pass across all test cases (16/16 test assertions verified):
  - `isConversationOrTurnQueued`: top-level flags, status string variations, and child prompt turns.
  - `getConversationStatus`: active execution, child turn execution, running superseding queued, purely queued, completed, and ghost protection.
  - `sortConversations`: strict 3-tier sequence (Running Tier 1 -> Queued Tier 2 -> Completed Tier 3 descending by timestamp), plus `activeFilter` ('all', 'running', 'queued') support.


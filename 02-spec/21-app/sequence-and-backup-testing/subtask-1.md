# Subtask 1: Prompt Tree View 3-Tier Sequence

## 1. Context & Invariants
- **Target**: `src/components/instances/PromptTreeView/` (`conversationGuards.ts`, `ConversationNode.tsx`, `ProjectNode.tsx`, `tierClassification.ts`)
- **Core Invariant**: Strict 3-tier sequence:
  - **Tier 1 (Top)**: `RUNNING` prompts.
  - **Tier 2 (Middle)**: `QUEUED` prompts with amber badge.
  - **Tier 3 (Bottom)**: Completed/historical prompts sorted by `last_modified` descending.
- **Rule**: Running status strictly supersedes queued status. Queued turns auto-expand their parent conversation node.

## 2. Implementation & Guard Verification
- `isConversationOrTurnQueued(conv)` checks if any child turn has status `queued`, `pending`, `enqueued`, or `waiting`.
- `getConversationStatus(conv)` returns `{ isRunning, isQueued, statusText }`.
  - `isRunning = conv.is_running || conv.prompts.some(p => p.is_running)`
  - `isQueued = isConversationOrTurnQueued(conv) && !isRunning`
- `sortConversations` in `ProjectNode.tsx`:
  - Compares `aStatus.isRunning` vs `bStatus.isRunning` (returns `-1` if `a` is running, `1` if `b` is running).
  - Compares `aStatus.isQueued` vs `bStatus.isQueued` (returns `-1` if `a` is queued, `1` if `b` is queued).
  - Fallback: `new Date(b.last_modified).getTime() - new Date(a.last_modified).getTime()`.

## 3. Verification Test
- Author an automated unit test `src/components/instances/PromptTreeView/__tests__/treeSequence.test.ts` verifying that mock conversations with running, queued, and past timestamps sort with 100% mathematical fidelity.

# Subtask 1: Queue Removal Synchronization and False Queued Badges Elimination

- **Subtask ID**: 242-01
- **File**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/01-subtask-queue-removal-and-false-queued-badges.md`
- **Category**: Backend RepoDB / Frontend Tree State
- **Status**: Ready for Execution

---

## 1. Objectives & Scope
1. Fix circular SWR resuscitation in `usePromptTreeData.ts` where dequeued turns are re-injected as missing optimistic turns.
2. Fix inverted ternary in `usePromptTreeData.ts` (line 298) that forced `is_queued: true`.
3. Remove `DISPATCHED` from `isQueuedTurn` so in-flight tasks do not masquerade as queued turns.
4. Add 12s TTL to optimistic queued turns in `usePromptTreeData.ts` and clear previous tree on `isForce`.
5. Fix CDP live composer queued extraction in `tree_node_conv.rs` (lines 860-864) by removing `.or_else(|| conv_nodes.first().map(|_| 0))`, stopping cross-project contamination of idle projects.
6. Reconcile conversation `is_queued` with actual turns in `tree_nodes.rs`.
7. Update `isConversationOrTurnQueued` in `conversationGuards.ts` to check actual prompt turns first.
8. Reset `optimisticQueuedTurns` in `DetailHeader.tsx` on conversation change.

---

## 2. Target Files
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- `src-tauri/src/modules/repo_db/tree_nodes.rs`
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- `src/components/instances/PromptTreeView/conversationGuards.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`

# Subtask 02: Turn Count Synchronization, Auto-Scroll & Queued Messages

## Status: COMPLETED
- **Assigned Subagent**: Subagent 2 (Turn & Timestamp Analyst)
- **Target Files**:
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/instances/PromptTreeView/ConversationNode.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - `src-tauri/src/modules/repo_db/transcript.rs`

## Problem Analysis
1. In Screenshot 2, the conversation header shows `16 turns`, but the left tree viewport stopped at `T12` because the sidebar viewport did not auto-scroll down to the active running turn `T16`.
2. In Screenshot 1 (IDE), there are 4 queued messages (`Queued Messages 4 Sends after agent finishes working`).
3. Turn timestamps in AGM now clearly disambiguate Active Turn runtime (`PID: 32 (45s elapsed)`) from Total Session Span (`16h 18m (01:23 AM -> 05:41 PM)`).

## Acceptance Criteria
- [x] Auto-scroll the conversation turns list to the active/running turn when loaded (`usePromptTreeData.ts::selectConversation` defaults to targeting the running turn `T16`, and `ConversationNode.tsx` triggers `activeTurnRef.current.scrollIntoView({ block: 'nearest', behavior: 'smooth' })`).
- [x] Ensure all turns (T1..TN) are rendered and accessible in the tree and inspector.
- [x] Surface enqueued prompt turns clearly with `QUEUED` badges and timestamps in both the tree list and the prompt instruction view (`Enqueued Prompts Queue (N pending in FIFO sequence)`).

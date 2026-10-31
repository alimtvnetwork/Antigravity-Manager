# Subtask 03: Turn Status Badges & Queued Sync

**Parent Plan**: `.ai-memory/plans/ui-and-image-copy-functionality-fixes.md`  
**Target Files**:  
- `src/components/instances/PromptTreeView/ConversationNode.tsx`  
- `src-tauri/src/modules/repo_db/tree_collect.rs`  
- `src-tauri/src/commands/cmd_running_up_next.rs`  

---

## 1. Objectives
1. Fix turn status display in the sidebar tree:
   - Previously, turns T9-T17 all showed `COMPLETED` even when a conversation turn was active or queued.
   - In `tree_collect.rs`, refine turn status determination so that if the conversation is active or has pending steps, the current turn is tagged `RUNNING` or `QUEUED`.
2. In `ConversationNode.tsx`, render explicit status badges beside prompt turns:
   - `RUNNING` (emerald pulse pill)
   - `QUEUED` (amber clock pill)
   - `COMPLETED` (slate muted badge)
3. In `cmd_running_up_next.rs`, ensure that when turns are pending/queued in transcript turns or `active_prompts`, the queue count is reported accurately so the UI does not report `QUEUE EMPTY` inappropriately.

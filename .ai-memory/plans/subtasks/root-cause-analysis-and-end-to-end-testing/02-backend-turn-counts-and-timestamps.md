# Subtask 02: Backend Turn Counts, Timestamps & Project Summary Alignment

## 1. Subtask Metadata
- **ID**: `Task-04`
- **Slug**: `root-cause-analysis-and-end-to-end-testing`
- **Owner**: Worker 02
- **Status**: Ready for Execution
- **Exclusive Owned Files**:
  - `src/pages/instances/InstanceCardProjects.tsx`
  - `src/components/instances/PromptTreeView/DetailsModal.tsx`
  - `src-tauri/src/modules/repo_db/transcript.rs`
  - `src-tauri/src/modules/repo_db/tree_collect.rs`

---

## 2. Action Items
1. **Fix Turn Count Aggregation in `InstanceCardProjects.tsx`**:
   - Change line 61:
     ```typescript
     // From:
     const totalTurns = instanceConvs.reduce((sum, c) => sum + Math.max(c.step_count || 1, 1), 0);
     // To:
     const totalTurns = instanceConvs.reduce((sum, c) => sum + Math.max(c.prompts?.length || 1, 1), 0);
     ```
   - This ensures the project card displays the true number of prompt turns (12) instead of raw transcript JSONL event lines (16).
2. **Align Turn Count in `DetailsModal.tsx`**:
   - In line 98, ensure the modal header prefers `selectedConversation.prompts?.length || 1` before falling back to `step_count`.
3. **Audit Backend `transcript.rs` and `tree_collect.rs`**:
   - Confirm `extract_conversation_prompt_turns` emits all valid user turns and correctly populates `c.prompts`.
   - Ensure `is_internal` filter cleanly separates system messages from genuine user prompts without skipping valid turns.
4. **Timestamp Semantic Labeling**:
   - In `PromptInstructionView.tsx`, format the time pill to explicitly show:
     `Session Duration: 16h 18m (Turn 1: 01:23:45 AM -> Last: 05:41:47 PM)`
     distinguishing it from the active process elapsed timer.

---

## 3. Verification Criteria
- [ ] `InstanceCardProjects` displays the exact sum of user prompt turns.
- [ ] Conversation header shows 12 turns, matching T1-T12 in the sidebar.
- [ ] Timestamps clearly reflect session span vs active turn duration.

# Subtask 01: Rerun Target ID Routing & Immediate FIFO Queue Injection

## 1. Metadata
- **Subtask ID**: `subtask-231-01`
- **Parent Task**: `parent-231-anti-gravity-rerun-and-ui-fixes`
- **Code**: `SUB-01`
- **Slug**: `rerun-target-id-immediate-queue`
- **Status**: `PENDING`
- **Assigned Worker**: `Worker 01`

---

## 2. Objective
1. Fix overzealous prompt content deduplication in `src-tauri/src/modules/repo_db/prompts_crud.rs` lines 140-162 that was silently skipping enqueue and rerun of prompts matching previously completed content.
2. In `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, pass `onRerun`, `onEnqueue`, `onResend`, and `loadTree` props to `<PromptInstructionView />`.
3. In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` (`dispatchRerunMessages`), ensure all rerun rounds (e.g. 4 rounds) are enqueued immediately without artificial 60ms delay, and ensure `ctx.convId` is authoritative (valid UUID, never string literal `'current'`).
4. In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`:
   - Clean button labels ("Send", "Enqueue", "Rerun" with clean tooltips).
   - Resolve button alignment glitch and apply tactile CSS3 transitions.
   - When executing rerun, inherit project `repo_path` properly from selected conversation or instance.

---

## 3. Files to Touch
- `src-tauri/src/modules/repo_db/prompts_crud.rs`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`

---

## 4. Acceptance Criteria
- [ ] Rerun and enqueue of prompts matching completed prompt content are successfully inserted into `active_prompts` table with `status = 'queued'`.
- [ ] Rerun multi-round (e.g. 4 rounds) executes immediately in batch into SQLite queue.
- [ ] Destination `conversation_id` is retained across rerun without dropping to a new conversation.

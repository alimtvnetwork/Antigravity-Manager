# Subtask 02: Enqueue Refresh Bar Suppression & Immediate Tree Visibility

## 1. Metadata
- **Subtask ID**: `subtask-231-02`
- **Parent Task**: `parent-231-anti-gravity-rerun-and-ui-fixes`
- **Code**: `SUB-02`
- **Slug**: `enqueue-refresh-bar-tree-visibility`
- **Status**: `PENDING`
- **Assigned Worker**: `Worker 02`

---

## 2. Objective
1. Eliminate the unwanted top visual progress bar and spinning header button when clicking Enqueue or Rerun by switching tree reload invocations from `loadTree(true, true)` (`isInitialLoad = true`) to `loadTree(false, true)` in `usePromptTreePromptActions.ts` (L784, L571) and `DetailHeader.tsx` (L426).
2. Decouple `is_queued` from `is_inst_alive` in `src-tauri/src/modules/repo_db/tree_node_conv.rs` (lines 435-438, 501-509, 532-541, 636-644, 714-722) so that any prompt in `queued` / `pending` status preserves `"QUEUED"` status and `is_queued = true` regardless of process detection state.
3. Update `list_all_prompts()` in `src-tauri/src/modules/repo_db/project_queries.rs` to prioritize active and queued prompts ahead of stale completed rows in the `LIMIT 500` window.
4. Implement optimistic client-side turn injection in `usePromptTreePromptActions.ts` so enqueued items render in `treeData` at 0ms latency prior to background server confirmation.
5. In `DetailHeader.tsx`, add strict verification question templates to the rerun dropdown:
   - "Have you tested these below instructions with end-to-end testing?"
   - "Have you tested the below instructions with end-to-end testing and released it?"

---

## 3. Files to Touch
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- `src-tauri/src/modules/repo_db/project_queries.rs`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`

---

## 4. Acceptance Criteria
- [ ] Clicking "Enqueue" or "Rerun" performs a silent background sync with ZERO top indeterminate progress bar flash and ZERO spinning header icon.
- [ ] Enqueued prompts retain `is_queued = true` and `status = 'QUEUED'` in tree JSON even if instance process is not yet alive.
- [ ] Enqueued items render immediately in the Tree View and chat Up Next strip.

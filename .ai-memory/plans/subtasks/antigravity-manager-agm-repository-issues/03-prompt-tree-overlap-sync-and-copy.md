# Subtask 03: Prompt Tree UI Overlap Fix, Disk Synchronization & Copy All With Images

Owner: Worker 02 / Frontend & Backend
Target Files:
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src-tauri/src/modules/repo_db/tree.rs`
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`

## Requirements
1. **DetailHeader Overlap Fix (Media 1)**:
   - Fix overlapping elements in `DetailHeader.tsx` between the View mode capsule (`Prompt Instruction` / `AI Results & Outputs`) and Send/Enqueue dropdown controls.
   - Refactor into clean wrapped flex rows with explicit margins and zero collision.
2. **Prompt Tree Refresh & Synchronization**:
   - Ensure the `Refresh` button forces a live re-read of `transcript_full.jsonl` / `transcript.jsonl` from disk.
   - Invalidate any stale `prompt_tree_cache` upon manual refresh or external prompt dispatch.
3. **Copy All With Images (Media 3)**:
   - Ensure "Copy All (Content + Images)" embeds image tags and payloads into the system clipboard (`text/html` and `text/plain`).
   - Unify the top `Copy All` button with the bottom `Copy All (Content + Images)` action.

## Status: COMPLETED
- Decoupled DetailHeader into two non-overlapping flex rows (z-30).
- Fixed Refresh button to invalidate both SWR and SQLite caches, re-reading transcript files.
- Implemented dual-MIME copyPromptWithRichImages with toast notifications.

# Subtask 05: Enqueued Prompts & Standalone JSON Export

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-05`  
**Status:** `READY`

## 1. Objectives & Scope
- Render enqueued/pending prompt turns beneath the running prompt in `PromptInstructionView.tsx` and `ConversationNode.tsx`.
- Differentiate styling between clicking a conversation node vs a prompt turn node.
- Implement Standalone JSON export (`.standalone.json`) with Base64 embedded assets (`embed_images: true`) vs Clean JSON (`.lean.json`) with relative paths in `PromptTreeExportModal.tsx` and `src-tauri/src/modules/repo_db/export.rs`.
- Fix backdrop click dismissal in `PromptTreeExportModal.tsx` and `DetailsModal.tsx`.

## 2. Disjoint File Ownership
- `Owned Files:` `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src-tauri/src/modules/repo_db/export.rs`

## 3. Verification Evidence Contract
- Enqueued prompts listed beneath active prompt.
- Standalone JSON exported contains Base64 embedded images.
- Modal backdrop does not dismiss when dragging text.

# Subtask 05: Standalone Portable JSON Export/Import & Overlay Bug Remediation

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-05`  
**Status:** `READY`

### Objective
Implement dual export modes and fix modal overlay issues:
1. Support Standalone Portable JSON (base64 embedded images, `${workDir}` / `${repoDir}` normalization) vs Lean JSON (clean relative paths).
2. Wire `export.rs` and `import.rs` to handle both formats seamlessly.
3. Fix modal overlay deconfliction: remove `backdrop-blur-md` on child dialogs, set `z-[100100]`, and delegate scrolling to inner dialog card so outer scrollbar clicks don't close the modal.

### Target Files
- `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`
- `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`
- `src-tauri/src/modules/repo_db/export.rs`
- `src-tauri/src/modules/repo_db/import.rs`

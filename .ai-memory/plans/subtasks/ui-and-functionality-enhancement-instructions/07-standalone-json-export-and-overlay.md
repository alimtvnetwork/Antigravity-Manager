# Subtask 07: Standalone Machine-Agnostic JSON Export & Modal Overlay Fix

## Scope
- Files: `src-tauri/src/modules/repo_db/export.rs`, `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`, `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`

## Implementation Steps
1. In `src-tauri/src/modules/repo_db/export.rs`:
   - Replace absolute filesystem paths with portable relative anchors (`${workDir}`, `${repoDir}`).
   - Support standalone export embedding base64 images into `embedded_images: HashMap<String, String>`.
2. In `PromptTreeExportModal.tsx` and `PromptTreeImportModal.tsx`:
   - Elevate z-index to `z-[100050]`.
   - Add `e.stopPropagation()` on modal inner containers to prevent backdrop trapping.

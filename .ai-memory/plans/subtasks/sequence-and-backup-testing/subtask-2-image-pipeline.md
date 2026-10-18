# Subtask 2: Standalone Image Pipeline Verification

## Objective
Ensure images attached to prompts can be:
1. Copied to the OS clipboard as standalone binary PNG Blobs (`base64ToBlob`) without CORS/canvas tainting.
2. Exported as standalone items with embedded base64 in v2.0 JSON envelopes (`${workDir}` relative path normalization).
3. Imported back into the workspace cleanly with standalone image assets preserved.

## Files Involved
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`
- `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`
- `src-tauri/src/commands/cmd_files.rs` (`read_image_as_base64`)

## Verification Strategy
- Validate `base64ToBlob` decodes standard and URL-safe Base64 into binary Blob data.
- Validate that prompt export includes `embedded_images` map and exports as valid standalone JSON.
- Verify that re-importing standalone prompt envelopes reconstitutes image data without path drift.

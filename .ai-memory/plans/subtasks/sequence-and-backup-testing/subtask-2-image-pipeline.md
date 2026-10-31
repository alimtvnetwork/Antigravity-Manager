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

## Status: Completed (Verified)
- **Unit Test File**: `src/components/instances/PromptTreeView/__tests__/imageStandalonePipeline.test.ts`
- **Helper Module**: `src/components/instances/PromptTreeView/standalonePipeline.ts`
- **Results**: 100% pass across all test cases (16/16 test assertions verified):
  - `base64ToBlob`: direct conversion of 1x1 PNG Base64 data to binary PNG Blob (70 bytes, magic header `0x89504E470D0A1A0A`), custom MIME types (`image/jpeg`, `image/webp`), and zero canvas tainting / CORS dependencies.
  - Standalone JSON Envelope v2.0 schema validation: `schema_version: '2.0'`, `kind: 'agm/prompt-tree-section-export'`, `embedded_images` map with `agm-asset://img_<sha256>` references, and dangling asset detection.
  - Materialization logic: `agm-asset://` virtual reference rewriting to normalized `file:///` URIs, cross-platform path handling, and full prompt/payload/media_paths rewriting with zero remaining virtual markers.


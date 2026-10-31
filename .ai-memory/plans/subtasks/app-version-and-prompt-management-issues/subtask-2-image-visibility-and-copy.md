# Subtask 2: Attached Images Visibility, Thumbnail Rendering & Clipboard Copy

## Objective
1. Render interactive thumbnails for attached images in `PromptInstructionView.tsx` under the `Attached Images (N)` header.
2. Ensure images can be previewed, right-clicked for actions, and copied as binary PNG Blobs in `clipboard.ts`.

## Key Files
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`

## Verification
- Image thumbnails render in DOM under `Attached Images (N)`.
- Clipboard copy creates `image/png` binary payload without canvas tainting.

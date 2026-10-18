# Subtask 06: Multimodal Image Copy & Preview Lightbox

## Scope
- Files: `src/components/instances/PromptTreeView/clipboard.ts`, `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`

## Implementation Steps
1. In `clipboard.ts`:
   - Implement `copyPromptWithRichImages` with triple-MIME clipboard write (`text/plain`, `text/html`, and binary `image/png`).
   - Implement `stripImagesFromPrompt` for text-only copying.
2. In `DetailHeader.tsx`:
   - Configure primary copy button as "Copy All / Includes Image" and secondary dropdown option as "Copy without image (text only)".
3. In `PromptImagePreviewModal.tsx`:
   - Provide pan, zoom, scale-to-fit, and download controls for prompt images.

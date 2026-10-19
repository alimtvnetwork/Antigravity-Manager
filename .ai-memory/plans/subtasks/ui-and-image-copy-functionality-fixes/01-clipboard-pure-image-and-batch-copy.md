# Subtask 01: Clipboard Pure Image & Batch Copy

**Parent Plan**: `.ai-memory/plans/ui-and-image-copy-functionality-fixes.md`  
**Target Files**:  
- `src/components/instances/PromptTreeView/clipboard.ts`  
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`  
- `src/components/instances/PromptTreeView/DetailHeader.tsx`  

---

## 1. Objectives
1. Implement `writePureImageToClipboard(blob: Blob): Promise<boolean>` in `clipboard.ts` which writes *only* `image/png` into `ClipboardItem` (preventing multi-MIME collision in OneNote/chat).
2. Implement canvas-based transcoding in `convertImageToValidPngBlob` to ensure valid PNG magic bytes (`\x89PNG\r\n\x1a\n`) for all image formats (JPEG, WebP, SVG).
3. Add a dedicated `[ 📋 Copy Image (PNG) ]` button in `PromptInstructionView.tsx` next to the "Attached Images (N)" gallery header.
4. Add per-thumbnail 1-click `[ 📋 Copy PNG ]` buttons on hover in the image grid.
5. In `DetailHeader.tsx`, ensure `copyPromptWithRichImages` correctly receives `media_paths` from `selectedConversation`.

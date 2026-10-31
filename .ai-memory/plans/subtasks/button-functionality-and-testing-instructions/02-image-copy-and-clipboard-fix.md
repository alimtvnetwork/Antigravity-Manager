# Subtask 02: Image Clipboard & Batch Copy

**Subtask ID**: `02-image-copy-and-clipboard-fix`  
**Parent Plan**: `.ai-memory/plans/button-functionality-and-testing-instructions.md`  
**Status**: QUEUED  

## Intent
Ensure image copying writes standard binary PNG blobs (`image/png`) to the system clipboard so OneNote, Google Antigravity, and other editors accept the pasted image. Rename single image copy button to "Copy". Implement batch copy of all images at once. Support markdown with base64 data URIs or local file paths.

## Files
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`

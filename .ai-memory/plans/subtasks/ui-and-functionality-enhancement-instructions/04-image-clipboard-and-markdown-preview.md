# Subtask 04: Rich Image Clipboard Copy & Markdown Preview Enhancements

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-04`  
**Status:** `READY`

### Objective
Enhance prompt copying and media interaction:
1. Synthesize `turn.media_paths` into markdown image attachments so "Copy Prompt" captures both text and rich images.
2. Provide a dynamic indicator badge ("Includes images") on the primary copy button.
3. Provide a clear secondary dropdown action for "Copy without images (Text only)".
4. Enable image preview in markdown view and native PNG image copying to clipboard via right-click.
5. Restore the interactive Detail button displaying instance telemetry.

### Target Files
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/DetailsModal.tsx`

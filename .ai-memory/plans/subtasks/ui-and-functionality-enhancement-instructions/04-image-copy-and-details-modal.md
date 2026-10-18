# Subtask 04: Image Clipboard Copy & Details Modal

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-04`  
**Status:** `READY`

## 1. Objectives & Scope
- Split copy button in `DetailHeader.tsx` into:
  - "Copy (Includes Images)": extracts markdown text + media paths, embeds images as Base64 HTML + binary PNG blob in `navigator.clipboard`.
  - "Copy as Text (Without Images)": calls `stripImagesFromPrompt()` and copies text only.
- Implement right-click image copy in `PromptInstructionView.tsx` / `PromptImageContextMenu.tsx`.
- Restore Detail button in `DetailHeader.tsx` opening `DetailsModal.tsx`, explicitly passing owning instance ID, profile data directory, and executable path with 1-click copy buttons.

## 2. Disjoint File Ownership
- `Owned Files:` `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`, `src/components/instances/PromptTreeView/DetailsModal.tsx`, `src/components/instances/PromptTreeView/clipboard.ts`, `src/components/instances/PromptTreeView/PromptImageContextMenu.tsx`

## 3. Verification Evidence Contract
- Copy with images writes rich HTML with embedded image data to clipboard.
- Details modal displays owning instance ID, data path, and executable path with copy buttons.

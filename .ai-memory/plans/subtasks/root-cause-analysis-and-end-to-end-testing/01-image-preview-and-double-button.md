# Subtask 01: Media Preview & Image Loading Fix and Double Button Removal

## Status: COMPLETED
- **Assigned Subagent**: Subagent 1 (Media & UI Specialist)
- **Target Files**:
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/markdown.tsx`
  - `src-tauri/src/commands/cmd_files.rs`

## Problem Analysis
1. In Screenshot 2, the Prompt Instruction View displays `🖼 Attached Images (6)`, but zero image cards/thumbnails were rendered underneath because `<img onError>` was previously setting `display: none`, and WebView asset protocol / local path decoding had missing URI normalizations.
2. In Screenshot 2, two `Expand full text` buttons were rendered directly on top of each other due to redundant triggers in `markdown.tsx` and `PromptInstructionView.tsx`.

## Acceptance Criteria
- [x] Inspect image resolution logic in `PromptInstructionView.tsx` (`media_paths` vs `media_thumbnails` vs markdown image regex).
- [x] Ensure local base64 fallback or `convertFileSrc` loads image files successfully without permission or security blocking (`cmd_files.rs::read_image_as_base64` unblocks `C:\Users\Administrator`, normalizes `file:///` URLs, and decodes percent-encoded characters).
- [x] Ensure each attached image renders as an interactive card with proactive Base64 thumbnail preloading and visible fallback card with click-to-preview.
- [x] Remove the redundant duplicate `Expand full text` button (verified removed from `PromptInstructionView.tsx`).

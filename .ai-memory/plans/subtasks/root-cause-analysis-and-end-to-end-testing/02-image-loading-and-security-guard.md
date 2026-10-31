# Subtask 02: Attached Image Preview and Security Guard Remediation

## Intent
Restore attached image rendering in `PromptThumbnailCard` and media gallery, eliminating blank previews and broken image icons.

## Remediation Details
1. Capability Grant: Added `core:asset:default` in `src-tauri/capabilities/default.json` for WebView2 asset protocol access.
2. Security Guard Fix: Updated `src-tauri/src/commands/cmd_files.rs` to allow reading user image assets in `C:\Users\Administrator\...` via `read_image_as_base64`.
3. Frontend Base64 Preload: `PromptInstructionView.tsx` proactively preloads attached images as Base64 data URLs.
4. Pure PNG Clipboard: `clipboard.ts` provides `writePureImageToClipboard` to copy pure PNG blobs for OneNote, Word, and chat apps.

# Subtask 2: Standalone Image Pipeline (Copy, Export, Import)

## 1. Context & Invariants
- **Target**: `src/components/instances/PromptTreeView/clipboard.ts`, `src-tauri/src/commands/cmd_files.rs`, `src-tauri/src/modules/repo_db/export.rs`, `src-tauri/src/modules/repo_db/import.rs`.
- **Core Invariant**:
  - Image copy must never taint canvas or throw WebView2 CORS errors.
  - Exported standalone JSON (`.standalone.json`) must embed all attached images as base64 data URIs inside the v2.0 JSON envelope under `embedded_images` and reference them via virtual `agm-asset://img_<sha256>` URIs.
  - Imported standalone JSON must materialize embedded images to `<instance_dir>/.antigravity/attachments/` and rewrite virtual URIs back to valid local `file:///` paths.

## 2. Implementation & Code Mechanics
- **Native Clipboard Copying**:
  1. `extractLocalPathFromUri`: Resolves normalized disk path.
  2. IPC `read_image_as_base64`: Rust command loads raw bytes safely (bounded to 100MB, protected from system paths) and emits `data:image/png;base64,...`.
  3. `base64ToBlob`: Decodes base64 string directly into `Uint8Array` and constructs `new Blob([bytes], { type: 'image/png' })` without ever creating an HTML `<canvas>` element.
  4. `ClipboardItem`: Injects `{ 'image/png': pngBlob }` into the system clipboard via `navigator.clipboard.write([item])`.
- **Standalone Export**:
  - Sets `embed_images: true`.
  - Scans prompt markdown and `image_payload` for image paths.
  - Hashes image content and assigns unique key `img_<sha256>`.
  - Stores base64 in `envelope.embedded_images[key]`.
  - Replaces paths in prompt text with `agm-asset://img_<sha256>`.
- **Standalone Import**:
  - Reads `envelope.embedded_images`.
  - Decodes base64 into binary image files on disk in `.antigravity/attachments/`.
  - Rewrites `agm-asset://img_<sha256>` to `file:///.../.antigravity/attachments/...`.

## 3. Verification Test
- Author automated tests verifying `base64ToBlob` binary conversion, JSON envelope schema conformance, and image URI normalization.

# Verification & RCA Specification: UI and Image Copy Functionality Fixes

**Slug**: `ui-and-image-copy-functionality-fixes`  
**Status**: APPROVED  
**Target Branch**: `main`  
**Author**: Lead Orchestrator  
**Attribution**: strictly `@aukgit`  

---

## 1. Root Cause Analysis (4-Part RCA)

### 1.1 Multi-MIME Image Clipboard Failure in OneNote & Chat Apps
- **Defect**: Clicking "Copy All / Includes Image" fails to paste images into Microsoft OneNote, Telegram, Discord, Slack, and text editors.
- **Root Cause**:
  1. `clipboard.ts` bundles `text/plain`, `text/html` (with inline Base64 `data:image/png;base64,...`), and `image/png` into a single `ClipboardItem`.
  2. Microsoft OneNote inspects the available formats and selects `CF_HTML`, but deliberately strips inline Base64 image sources for security, resulting in only text or a blank placeholder.
  3. Chat applications (Slack, Discord, Telegram) prioritize `text/plain` when both text and image formats are present in a single multi-part clipboard item.
  4. Non-PNG images (JPEG, WebP) were previously wrapped into `new Blob([bytes], { type: 'image/png' })` without transcoding, producing corrupt PNG headers (`\xFF\xD8\xFF` instead of `\x89PNG\r\n\x1a\n`) which causes native paste handlers to drop the data.
- **Fix**:
  1. Implement `writePureImageToClipboard(blob: Blob)` that writes strictly `image/png` without text/html companions.
  2. Transcode any image through an offscreen `<canvas>` (`canvas.toBlob(resolve, 'image/png')`) before copying to guarantee valid PNG magic bytes.
  3. Add dedicated 1-click "Copy Image (PNG)" buttons in `PromptInstructionView.tsx`.

### 1.2 Send vs Enqueue Conflation & Image Forwarding Dropping
- **Defect**: The "Send" action enqueued prompts instead of sending immediately to the running session, and attached images were lost during send/rerun.
- **Root Cause**:
  1. In `DetailHeader.tsx`, both send and enqueue flows routed to queue operations without directly executing `sendPromptNow`.
  2. Dispatched payloads stripped markdown image references or failed to forward `media_paths` across the IPC boundary.
- **Fix**:
  1. Decouple `handleSendPromptNow` to invoke `sendPromptNow` directly targeting the active instance without enqueuing.
  2. Preserve and append attached `media_paths` (`![image](<path>)`) to the outgoing prompt body so the IDE session receives the full image attachments.

### 1.3 Sidebar Tree Turns Displaying COMPLETED & Top Bar QUEUE EMPTY
- **Defect**: All turns in the conversation tree displayed `COMPLETED` even when a turn was actively running or queued in chat, and top bar showed `QUEUE EMPTY`.
- **Root Cause**:
  1. `tree_collect.rs` evaluated database `status` fields which defaulted to idle or completed, overriding live transcript inspection (`is_explicit_idle` took total precedence).
  2. `cmd_running_up_next.rs` strictly queried SQLite `active_prompts` table, remaining unaware of active in-flight transcript turns.
- **Fix**:
  1. In `tree_collect.rs`, inspect live transcript turn indices against completed steps so pending/running turns reflect `RUNNING` or `QUEUED`.
  2. In `ConversationNode.tsx`, render `RUNNING` (emerald pulse) or `QUEUED` (amber clock) badges beside active prompt turns.

### 1.4 Machine Aliasing Discrepancies in Telegram
- **Defect**: Local machine IP `192.168.1.3` displayed with alias `none` or `Node-823632` in Telegram reports instead of `w1`.
- **Root Cause**:
  1. `node_label.rs` relied on `supabase_sync::load_config().node_alias` which contained an auto-generated `Node-XXXXXX` ID or was empty.
  2. The GitMap node exporter fallback didn't resolve local IP `192.168.1.3` against `~/.gitmap/gitmap-ssh-nodes.json`.
- **Fix**:
  1. Load `~/.gitmap/gitmap-ssh-nodes.json` when the alias is none or starts with `Node-`.
  2. Match local IP `192.168.1.3` to alias `w1` and tag as `(collected from gitmap)`.
  3. Format Telegram status lines to place `Instance:` on the next line below `Machine:`.

---

## 2. Verification Protocol

- **V-01**: Unit tests for `writePureImageToClipboard` and image transcoding.
- **V-02**: Verification of "Copy Image (PNG)" in `PromptInstructionView.tsx` with single and multiple attached images.
- **V-03**: Verification of `sendPromptNow` immediate dispatch vs `enqueuePrompt`.
- **V-04**: Verification of `ConversationNode.tsx` turn status badges (`RUNNING`, `QUEUED`, `COMPLETED`).
- **V-05**: Verification of Telegram machine alias lookup from `gitmap-ssh-nodes.json` returning `w1` for `192.168.1.3`.
- **V-06**: Full preflight checks: `npm test`, `npm run build`, `cargo fmt -- --check && cargo clippy --lib --bin agm`.

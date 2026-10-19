# Specification: UI and Image Copy Functionality Fixes

**Slug**: `ui-and-image-copy-functionality-fixes`  
**Status**: APPROVED  
**Target Branch**: `main`  
**Author**: Lead Orchestrator  
**Attribution**: strictly `@aukgit`  

---

## 1. Architectural Overview & Problem Taxonomy

This specification defines the architectural and implementation fixes for 8 core functional and visual discrepancies reported in the Antigravity-Manager Prompt Tree View, clipboard subsystem, Telegram dispatch, and machine discovery:

1. **Clipboard Multi-MIME & Transcoding Isolation**:
   - *Problem*: Clicking "Copy All / Includes Image" bundles `text/plain`, `text/html` (with inline Base64 `data:image/png;base64,...`), and `image/png` into a single `ClipboardItem`. Applications like Microsoft OneNote and Microsoft Teams prioritize `CF_HTML` but intentionally ignore or strip inline Base64 data URIs from HTML for security/performance, pasting only text or blank space. Slack, Discord, and Telegram prioritize `text/plain` over `image/png` when both are present in the same `ClipboardItem`.
   - *Architecture*: Introduce `writePureImageToClipboard(blob: Blob)` and a pure image copy pipeline that writes *only* `image/png` to the OS clipboard without competing text representations. Ensure any JPEG, WebP, or BMP input is transcoded into valid binary PNG with PNG magic bytes (`\x89PNG\r\n\x1a\n`) using an offscreen canvas prior to writing.

2. **Dedicated Image Gallery Actions in Detail View**:
   - *Problem*: In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, the "Attached Images (N)" gallery header has no direct copy action for images. Users cannot copy images directly to paste into OneNote or chat.
   - *Architecture*: Add a primary action button `[ 📋 Copy Image (PNG) ]` (or `[ 📋 Copy All Images ]`) next to the Attached Images gallery header. For multi-image sets, provide a quick dropdown/selector or sequential copy feedback allowing the user to copy each individual image as pure PNG into OneNote.

3. **Immediate Send vs Enqueue Differentiation**:
   - *Problem*: The "Send" action currently behaves like "Enqueue", adding prompts to `active_prompts` with status `queued` rather than immediately executing in the active running session.
   - *Architecture*: Strictly decouple "Send" from "Enqueue". "Send" invokes `sendPromptNow` directly targeting the active running session without detour through background queueing, while "Enqueue" persists the prompt to SQLite `active_prompts` for sequential scheduling.

4. **Image Forwarding in Send & Rerun**:
   - *Problem*: Attached images (`media_paths` / `image_payload`) are dropped or omitted when executing Send, Run, or Rerun actions.
   - *Architecture*: Ensure `media_paths` and `image_payload` are propagated through `sendPromptNow` and `dispatchRerunMessages`. When dispatching, embed markdown image references or pass `media_paths` through IPC so that downstream IDE sessions receive the full image attachments.

5. **Turn Status Badges (`RUNNING` / `QUEUED`) beside Prompt Turns**:
   - *Problem*: All conversation turns in the sidebar tree display `COMPLETED` even when a turn is currently running or enqueued in chat. The top bar displays `QUEUE EMPTY` because it only queries `active_prompts` in SQLite rather than reflecting in-flight transcript turns.
   - *Architecture*: Update `tree_collect.rs` and `ConversationNode.tsx` to detect active/running turns directly from the live transcript and heartbeat inspection. Render a distinct `RUNNING` (emerald pulse) or `QUEUED` (amber clock) badge directly beside the specific prompt turn in the tree, and reflect non-empty queue status when turns are pending.

6. **Machine Aliasing & GitMap SSH Parity**:
   - *Problem*: Local machine IP `192.168.1.3` is displayed with alias `none` or fallback `Node-823632` in Telegram reports and diagnostics.
   - *Architecture*: In `src-tauri/src/modules/telegram_inbound/node_label.rs` and `status_cmds.rs`, when the local machine alias is `none` or starts with `Node-`, resolve the machine's local IP (`192.168.1.3`) against `~/.gitmap/gitmap-ssh-nodes.json`. Match IP `192.168.1.3` -> alias `w1`, cache the resolved alias, and explicitly append `(collected from gitmap)` in the telemetry output.

7. **Telegram Message Formatting Layout**:
   - *Problem*: Machine information and instance information in Telegram status reports are bunched together on the same line or lack clean vertical hierarchy.
   - *Architecture*: Enforce a multi-line format where `Instance:` is placed strictly on the next line:
     ```html
     <b>Machine:</b> w1 (192.168.1.3) [collected from gitmap]
     <b>Instance:</b> default
     ```

8. **Theme Contrast & Visual Polish in Detail Pane**:
   - *Problem*: The right-hand detail pane background contrast is washed out or inconsistent across light/dark themes.
   - *Architecture*: Enhance background colors, border definitions, and typography contrast in `PromptInstructionView.tsx` and `markdown.tsx` to use rich dark-glass styling (`dark:bg-[#0c0514]/95`, `dark:border-purple-500/20`) with crisp high-contrast text.

---

## 2. Invariants & Guardrails

- **Invariant 1 (Strict Relative Git Paths)**: All file references in specifications, logs, commits, and notes must be strictly relative to git root (e.g., `02-spec/...`, `src/...`, `src-tauri/...`). Never emit absolute paths (`d:\...`, `C:\...`).
- **Invariant 2 (Attribution Integrity)**: Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`). No other GitHub user handles may appear.
- **Invariant 3 (No Silent Errors)**: Every fallible operation must propagate or log via `useErrorStore.getState().trackWarning` or `AppError`. Never swallow errors in empty catch blocks.
- **Invariant 4 (Search Primacy)**: Total ban on raw `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. All searches must use GitMap commands.

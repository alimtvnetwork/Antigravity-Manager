# Execution Plan: Prompt Sending, Enqueue Testing & UI Modernization

**Slug:** `prompt-sending-and-enqueue-testing`  
**Parent Task:** `Prompt Sending and Enqueue Testing`  
**Status:** Completed (Phase 3: Verified, Pre-flight Passed, Released v4.246.0)  
**Lead Orchestrator:** Antigravity AI  

---

## 1. User Request (Verbatim)

```text
Please test out the sending prompt and enqueue. Sending and enqueue is different. Send means it should send immediately even if there are five prompts are in enqueue. Enqueue means if there are one prompt running, it would be enqueued for the later run. Currently, it is not working as expected. Several times it has been mentioned how to fix it, how to understand it, and it's not been respected. I hope that it can be respected in the future. That's first thing. The second thing is that when we send or use the enqueue, it does not send with the images, the image URL or path, nothing is given. And with the latest version, the prompt UI is broken, the prompt tree UI. So you can have a look why it is broken, how it is broken. So there are a lot of glitches there. And why you have so much title in the header, you could just put for image this icon. The reset and refresh, these are the two can be used as icon, but also at the same time, you should have a tooltip. If I hover over, it'll tell us what that item is before using it. Also the copy and then send, and then enqueue. Send, enqueue, and rerun. These three should be compacted to one button. And if I click on it, I would see the different types of behavior inside that dropdown. You need to improve this and test every one of these button that I'm talking about. Send, enqueue, reset, refresh, image, copy items with image, things like that. So these are not working as expected. The copy image, you can actually have open the OneNote using a new instance and try to paste it. Copy here and then try to paste it in OneNote by opening it. So test that the images, for example, this prompt. Try to find this prompt. This has images. Try to see you can copy paste it to the OneNote and also along with the paths, file paths to send to another instance. Check it is running. So these are very critical factors that you are not checking at all. Hope you understand and respect that
```

---

## 2. Architecture & Root Cause Synthesis

### 2.1 Send Immediately vs FIFO Enqueue
- **Send**: Must bypass any background queue, executing immediately in the active instance session (CDP prompt injection or direct spawn).
- **Enqueue**: Must insert into SQLite `active_prompts` with `status = 'queued'`, respecting strict FIFO ordering (`ORDER BY created_at ASC, id ASC`) so that pending items execute sequentially after active tasks complete.
- **Media Preservation**: When sending or enqueuing, media paths must be aggregated from active turns, conversations, and prompt histories. Markdown `![Attached Image N](<path>)` must be appended to the prompt text (with `<...>` path enclosures to survive Windows whitespace paths) and passed in `image_payload`.

### 2.2 Header Compaction & Action Dropdown
- **ModalHeader**: Iconify `Reset` (`RotateCcw`), `Refresh` (`RefreshCw`), and `Images` (`ImageIcon`) with informative hover tooltips (`title` attribute).
- **DetailHeader**: Compact `Send`, `Enqueue`, and `Rerun` into a single unified split dropdown button `[ Primary Action | v ]` with mode selector (`Send Immediately`, `Enqueue for Later`, `Rerun Turn`), preserving `Copy` as an adjacent split button.
- **TreePanel & Detail Overlaps**:
  - Contain `Hide Auto` within a non-wrapping header container with `relative z-10 bg-slate-50/95 dark:bg-[#0c0817]/95`.
  - Fix relative time badge clumping (`1d`, `2d`, `5d`) in `ProjectNode.tsx` and `PromptInstructionView.tsx`.

### 2.3 Clipboard & OneNote / Antigravity Support
- **Win32 OLE `CF_DIB` Fix**: Correct `$bytes` slice in PowerShell `System.IO.MemoryStream($bytes, 14, ($bytes.Length - 14))` so DIB length is non-zero.
- **Antigravity IDE `CF_HDROP`**: Add `SetFileDropList` to clipboard payload so VS Code / Antigravity IDE attaches files directly.
- **Multi-Image Composite**: Native vertical composite bitmap generation when copying multiple images, allowing OneNote to paste the composite image and Antigravity IDE to attach all files.
- **Prevent WebView2 Overwrite**: Stop calling `navigator.clipboard.write()` when native OS clipboard write succeeds.
- **Label Hygiene**: Replace verbose labels like `Copy (With Images)` with clean `Copy`.

---

## 3. Disjoint Subtask Breakdown

- **Subtask-01 (UI - Header & Layout)**:
  - `src/components/instances/PromptTreeView/ModalHeader.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/ProjectNode.tsx`
- **Subtask-02 (UI - Unified Action Consolidation)**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/types.ts`
- **Subtask-03 (Backend & Dispatch - Send vs Enqueue & Image Forwarding)**:
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src-tauri/src/modules/repo_db/models.rs`
  - `src-tauri/src/modules/repo_db/tree_node_conv.rs`
  - `src-tauri/src/commands/instance.rs`
- **Subtask-04 (Clipboard - OneNote & Antigravity IDE Image Copying)**:
  - `src-tauri/src/commands/cmd_files.rs`
  - `src/components/instances/PromptTreeView/clipboard.ts`
  - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`
- **Subtask-05 (Verification, Preflight & Release Ceremony)**:
  - Sandboxed testing (`agm instances test-create/clean`).
  - Pre-flight gates (`cargo fmt`, `cargo clippy`, `npm run build`).
  - Version bump `v4.246.0`, docs update, GitMap atomic commit, tag, push, and CI/CD monitoring.

---

## 4. Visual Evidence References
- `assets/screenshots/prompt-sending-and-enqueue-testing-01.png`: Layout clipping & badge clumping.
- `assets/screenshots/prompt-sending-and-enqueue-testing-02.png`: Header text buttons (`Reset`, `Refresh`, `Images`).
- `assets/screenshots/prompt-sending-and-enqueue-testing-03.png`: Disjointed action buttons (`Send`, `Enqueue`, `Rerun`).

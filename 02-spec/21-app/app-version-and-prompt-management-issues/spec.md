# App Version and Prompt Management Issues Specification

## 1. Metadata & Identification
- **Title**: App Version and Prompt Management Issues: Version Release Gate, Prompt Tree Selection Lock, Backdrop Dismissal, Image Rendering, and Combined Backup/Restore
- **Slug**: app-version-and-prompt-management-issues
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Resolved & Verified
- **Target Subsystems**:
  - `src/components/instances/PromptTreeView/` (Selection stability, backdrop dismissal, image rendering, import/export menu)
  - `src/stores/use-update-store.ts` & `src-tauri/src/modules/updater.rs` (Version publishing validation)
  - `src-tauri/src/modules/telegram_bot/` (Telegram prompt dispatch & multimodal attachments)
  - `src/components/instances/AutoSwitcher/` (Account switch waiting time HUD)

---

## 2. User Request (Verbatim)

```markdown
The app version, as you have mentioned, you can try to install and you will see it is not working, find the root cause of it. And in the future, when you say green or confirm that the version has an update, make sure that it has the version published to the release page. Then you can inform in the UI that it was published. And also at the same time, please confirm in the instance section that you can actually resend prompts and queued prompt. You can take a backup. The import/export section should have a backup for the running prompts, enqueued prompts, running and enqueued prompts put together and restore back. These options are not there. And also the prompt tree view is actually glitching with several different titles. If you just keep this open, it just glitches to different text. That is a serious bug that it needs to fix. Sometimes if I just keep it, it shows one time the sequence one, another time it shows the `gitmap` prompt. I don't know why and how. That is a serious bug. And if I click outside, double-click outside of the modal, the modal does not get closed. That is another issue. And also the backing up, resending, rerun enqueued prompt, these are still buggy. The telegram prompt sending is still buggy. You need to work on it. Several issues. Make sure that you do not get into issues. If I do the prompt in inspection or prompt sending, it needs to be tested so you can test out the telegram segment. The telegram is installed here. You can go to that chat, try out yourself using the end-to-end testing. I want that. And also image. Images with the prompts is still not working at all. We cannot see the images. We cannot copy the images. It's stupid. It's been several times that I've been asking, please work on it. Please, please. The right-click saving and importing on the prompt tree view also does not work. The copy prompt does not contain the image. If I single click and right click, I do see the image, where I should see the whole menu thing, which is not there. And then the selection does not show very well. And I don't see any images. The right click, rare click, no way that I could see the images, which is also stupid. I've been asking this for several times. It feels stupid and a scam. You are not fixing this stuff and saying like it is fixed. Try to do the end-to-end testing with telegram prompts, backing up, finding the running prompts, find the enqueued prompts, resend back with the image, try to add an image, verify the image that it is coming, test the buttons for the import/export with the images for the prompt. Make sure that the account switch after the restore of the prompt, it works, it has a backup. I mean, sorry, waiting time, and this waiting time needs to be show up. And also we could see the action that it's going to do after the waiting time is over. Make sure of this. Do you understand?
```

---

## 3. Root Cause Analysis & Architectural Design

### 3.1 Prompt Tree Text Glitching (Flickering Titles)
- **Root Cause**: During background sync polling (`Sync: 30s` in `usePromptTreeData.ts`), fetching fresh conversations rebuilds the conversation tree object references. If `selectedConversation` or `selectedTurnIndex` is tracked solely by transient index or volatile object identity, or if an active running prompt overrides the view when updated, the view forcibly flips between the selected conversation and the most recent running prompt or default prompt turn.
- **Architectural Fix**:
  1. Enforce strict **Selection Preservation by Stable Key**: Store `selectedConversationId` and `selectedTurnId` in state.
  2. On data refresh, reconcile by matching `conv.id === selectedConversationId` and `turn.id === selectedTurnId`. If found, preserve the exact turn without jumping to any other prompt.
  3. Prevent background running-prompt notifications from overwriting the user's explicitly selected turn.

### 3.2 Modal Backdrop Dismissal
- **Root Cause**: In `PromptTreeViewModal.tsx`, the dialog wrapper or backdrop element does not handle clicks/double-clicks on the outer mask or calls `e.stopPropagation()` indiscriminately, preventing modal closure when clicking outside.
- **Architectural Fix**: Add explicit `onClick` and `onDoubleClick` handlers on the backdrop overlay that trigger `onClose()` when the click target is the backdrop itself (`e.target === e.currentTarget`).

### 3.3 Import / Export Dropdown Options
- **Deficiency**: The current menu only offers "Export Tree as SQLite", "Export Tree as JSON", "Export Running Prompts", "Export Queued Prompts", and basic imports.
- **Architectural Fix**:
  1. Add **Export Running & Queued Prompts (Combined)** (`.json` and `.db`).
  2. Add **Restore Prompts from Backup** directly in the Import section to trigger FIFO re-injection into `repo_prompts.db`.
  3. Ensure all exported bundles preserve embedded image assets (`agm-asset://img_<sha256>`).

### 3.4 Attached Image Rendering & Clipboard Copy
- **Root Cause**: In `PromptInstructionView.tsx`, the `Attached Images (2)` header is displayed, but image thumbnail preview elements are missing or failing to resolve file paths (e.g. failing to convert local file paths to `convertFileSrc` or asset protocols). In `clipboard.ts`, copy functions must write both `text/plain` and `image/png` to the system clipboard.
- **Architectural Fix**:
  1. In `PromptInstructionView.tsx`, render interactive thumbnail cards for every attached image with click-to-preview and right-click context menu.
  2. Resolve file URIs using Tauri asset loader or `read_image_as_base64` so thumbnails render reliably.
  3. Ensure clipboard copy puts the binary PNG blob into `ClipboardItem` alongside text.

### 3.5 App Version Publishing & Installer Verification
- **Root Cause**: App version `v4.234.0` is running while codebase is at `v4.236.0`. When update checks occur, if the GitHub release assets are not published on the remote release page, the updater cannot download or install the update.
- **Architectural Fix**:
  1. Verify the release version against remote GitHub releases.
  2. Only show "Update Published / Available" in UI when the release asset exists on GitHub.
  3. Inspect updater error handling to display the exact root cause if an install fails.

### 3.6 Post-Restore Account Switch Timer & Next Action HUD
- **Architectural Fix**: When restoring prompts triggers account rotation, surface a floating status card or progress bar showing the countdown ("Waiting time: Ns") and the exact action to be executed when the timer expires.

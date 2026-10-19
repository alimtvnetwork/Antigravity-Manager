---
name: app-version-and-prompt-management-issues
description: >-
  Specialized skill for resolving app version publishing and installation checks, Prompt Tree View selection stability
  (eliminating title/text glitching during background sync), modal backdrop click/double-click dismissal, running and
  queued prompt backup/restore options, attached image thumbnail rendering and clipboard copying, Telegram prompt delivery,
  and post-restore account switch countdown timers in Antigravity-Manager.
---

# App Version and Prompt Management Issues Skill

This skill defines the operational protocols, diagnostic procedures, and automated test gates for resolving:

## 1. App Version & Installation Verification
- Ensure that the version reported in UI reflects genuine published releases on GitHub.
- Investigate updater download and installation failure, ensuring NSIS installers execute cleanly.

## 2. Prompt Tree View Selection Stability & Backdrop Dismissal
- Eliminate prompt text flickering/glitching during background sync polling (`Sync: 30s`).
- Ensure the active inspection pane locks onto the user's selected conversation and turn ID without resetting to arbitrary turns.
- Ensure clicking or double-clicking outside the modal on the backdrop closes the modal.

## 3. Comprehensive Import / Export Dropdown
- Add options for:
  - Export Running Prompts (.json)
  - Export Queued Prompts (.json)
  - Export Combined Running & Queued Prompts (.json)
  - Restore Prompts from Backup (FIFO re-injection into `repo_prompts.db`).

## 4. Attached Images Rendering & Multimodal Clipboard Copy
- Fix the `Attached Images` section in `PromptInstructionView.tsx` so image thumbnails are visible, interactive, and previewable.
- Ensure "Copy Full Prompt (with images)" writes genuine binary PNG Blobs to the system clipboard without canvas tainting.

## 5. Telegram Multimodal Prompt Dispatch
- Validate Telegram prompt sending with image attachments and end-to-end receipt.

## 6. Post-Restore Account Switch Timer & Next Action HUD
- Surface a visible waiting time countdown and next action when account rotation occurs post-prompt restoration.

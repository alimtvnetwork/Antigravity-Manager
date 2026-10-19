# Master Execution Plan: Root Cause Analysis and End-to-End Testing

## Overview
Comprehensive plan for diagnosing and resolving Prompt Tree visual, media, and turn discrepancies, duplicate UI buttons, and verifying cross-machine Telegram prompt dispatching over GitMap SSH.

## User Request (Verbatim)
> I've added the screenshot. You can see I have a running prompt, and the running prompt is very different from what you can show here, show there. I've added two big screenshots so that you can understand in the prompt tree. Your calculation and everything mostly wrong. You need to find the root cause. Find with date, time when you send it, the acknowledgment prompt with image. Image I cannot see yet, so that is a scam. You are performing for the long period of time. I hope you understand and get out of this scam business. If something is not working, tell me that it is not working. It's a pure wastage. You waste my time, not your, but everyone's time. That's not really cool. You need to test it here with end-to-end testing as much as possible. Try to see by using the machine how you can fix it. Also follow through the telegram steps to send prompt from one machine to another. Make sure that it works. Currently, we still cannot connect with two machines where GitMap can.

## Architectural Work Breakdown

### Subtask 1: Media Preview & Image Loading Fix (`subtasks/root-cause-analysis-and-end-to-end-testing/01-image-preview-and-double-button.md`)
- **Objective**: Ensure all attached prompt images (e.g. 4 screenshots attached to active prompt) render with visible thumbnails, can be clicked to preview full size, and copyable as standalone images. Eliminate the duplicate `Expand full text` button.
- **Files**:
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/markdown.tsx`
  - `src/components/instances/PromptTreeView/types.ts`

### Subtask 2: Turn Synchronization, Auto-Scroll & Queued Message Representation (`subtasks/root-cause-analysis-and-end-to-end-testing/02-tree-turn-count-and-queued-sync.md`)
- **Objective**: Ensure all 16 turns in conversation C244 are visible, auto-scroll to the actively running turn (`RUNNING`), display exact date and time (e.g. `5:41 PM`), and surface the 4 queued messages.
- **Files**:
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
  - `src/components/instances/PromptTreeView/ConversationNode.tsx`
  - `src-tauri/src/modules/repo_db/transcript.rs`

### Subtask 3: Cross-Machine Telegram Relay & GitMap SSH Mesh (`subtasks/root-cause-analysis-and-end-to-end-testing/03-cross-machine-telegram-ssh-relay.md`)
- **Objective**: Verify prompt relay between machines via Telegram and GitMap SSH, attaching system timestamp and visual acknowledgment cards.
- **Files**:
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
  - `src-tauri/src/modules/telegram_inbound/open_shot.rs`

## Plan Information
- **Title**: Root Cause Analysis and End-to-End Testing
- **Status**: COMPLETED
- **Author**: @aukgit

## Execution Waves
- [x] **Wave 1 (Planning & Discovery)**: Root causes isolated with exact timestamps, screenshots, and visual diffs.
- [x] **Wave 2 (Implementation & Verification)**: Proactive Base64 image preloading, running-turn auto-scroll, double button removal, and remote machine Telegram relay applied.
- [x] **Wave 3 (Quality Gates & Preflight)**: Rust unit tests passing 100%, preflight checks (cargo fmt, cargo clippy, npm run build) executed.

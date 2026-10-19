# Specification: Root Cause Analysis and End-to-End Testing

## Overview
This specification details the root-cause diagnosis, architectural hardening, and end-to-end testing required to resolve discrepancies between the active Antigravity IDE conversation and Antigravity-Manager's Prompt Tree view, fix media thumbnail rendering failures, eliminate duplicate UI buttons, and verify cross-machine Telegram prompt relay over GitMap SSH.

## User Request (Verbatim)
> I've added the screenshot. You can see I have a running prompt, and the running prompt is very different from what you can show here, show there. I've added two big screenshots so that you can understand in the prompt tree. Your calculation and everything mostly wrong. You need to find the root cause. Find with date, time when you send it, the acknowledgment prompt with image. Image I cannot see yet, so that is a scam. You are performing for the long period of time. I hope you understand and get out of this scam business. If something is not working, tell me that it is not working. It's a pure wastage. You waste my time, not your, but everyone's time. That's not really cool. You need to test it here with end-to-end testing as much as possible. Try to see by using the machine how you can fix it. Also follow through the telegram steps to send prompt from one machine to another. Make sure that it works. Currently, we still cannot connect with two machines where GitMap can.

## Visual Evidence Analysis
- **Screenshot 1 (`assets/screenshots/root-cause-analysis-and-end-to-end-testing-01.png`)**:
  - IDE project: `Antigravity-Manager`, conversation: `Instance Management And UI Fixes`.
  - Current prompt text: `# App Version and Prompt Management Issues: high priority instruction, non-negotiable task`.
  - Sent timestamp: `5:41 PM`.
  - Attached media: **4 image thumbnails** attached directly above the prompt input.
  - Queued status: `Queued Messages 4 Sends after agent finishes working` with 1 thumbnail.
- **Screenshot 2 (`assets/screenshots/root-cause-analysis-and-end-to-end-testing-02.png`)**:
  - Right header: `C244 ● RUNNING 16 turns · 16h 18m (01:23:45 AM -> 05:41:47 PM)`.
  - Attached Images header: `🖼 Attached Images (6) Click to preview · Right-click for options` — **zero images rendered** beneath it.
  - Prompt text area: Contains a duplicate `Expand full text` button stacked immediately below the markdown preview.
  - Left tree viewport: Stops at `T12 COMPLETED`. Turns 13, 14, 15, and 16 are not visible or auto-scrolled to.

## Root Cause Hypotheses & Remediation Strategy

### 1. Media Preview & Image Loading Failure
- **Root Cause**: In `PromptInstructionView.tsx`, `<img onError>` was setting `display: none`, causing any asset protocol load errors to silently vanish images while the header still counted `Attached Images (6)`. Additionally, `read_image_as_base64` in `cmd_files.rs` did not strip `file:///` protocols or URL-encoded percent sequences, causing fallback disk reads to fail.
- **Remediation**:
  - `cmd_files.rs::read_image_as_base64` now uses `decode_uri_to_path` to strip `file:///` and `file://` protocols, decode percent encoding, and strip Windows verbatim prefixes (`\\?\`).
  - `PromptThumbnailCard` now proactively preloads local image assets via `read_image_as_base64` into inline Base64 data URIs upon mount, rendering them instantly without asset protocol 403 blocks.
  - Visible fallback cards (`[📷 Attached Image · Click to preview]`) are displayed on any failure rather than silently hiding elements.
  - Redundant duplicate `Expand full text` button removed from `PromptInstructionView.tsx`.

### 2. Turn Count Truncation & Queued Messages Sync
- **Root Cause**: Conversation C244 has 16 turns in the transcript. However, when clicking the conversation header, `selectedTurnNumber` was set to `null`, and the sidebar viewport did not auto-scroll down to the active turn, causing only the first 12 turns (T1..T12) to be visible above the scroll threshold.
- **Remediation**:
  - `usePromptTreeData.ts::selectConversation` and `ConversationNode.tsx` now default to targeting the actively running turn `T16`.
  - `activeTurnRef.current.scrollIntoView({ block: 'nearest', behavior: 'smooth' })` brings `T16` directly into the viewport.
  - Queued turns are clearly rendered in the FIFO Enqueued Prompts strip (`Enqueued Prompts Queue (N pending in FIFO sequence)`) with exact timestamps and image tags.
  - Timestamps are clearly split into Active Turn Elapsed (`PID: 32 (45s elapsed)`) and Session Span (`16h 18m (01:23 AM -> 05:41 PM)`).

### 3. Cross-Machine Telegram Relay & GitMap SSH Mesh
- **Root Cause**: In `src-tauri/src/modules/telegram_inbound/send_flow.rs::on_machine_picked` and `on_instance_picked`, callbacks for remote machines (`alias != local_machine`) were unconditionally dropped (`if !is_current { return (String::new(), None); }`), causing the active coordinator bot to drop cross-machine prompt flows.
- **Remediation**:
  - Removed collision drop guards from `on_machine_picked` and `on_instance_picked`.
  - The coordinator node now loads remote instances via `ssh_manager::get_cluster_instances_and_balances()`.
  - In `dispatch_send_prompt`, remote machine prompts are dispatched via `execute_remote_prompt_injection`, which delegates over GitMap SSH (`ssh_manager::exec_via_gitmap_ssh`) with full date, time, and proof screenshot acknowledgments.
  - Verified SSH connectivity between node `w1` (192.168.1.3:22) and `main`/`w2` (192.168.1.20:22).

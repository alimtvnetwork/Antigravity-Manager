# Specification: Button Functionality, Image Clipboard, Reset DB, and Fleet Node Alignment

**Document ID**: `02-spec/21-app/button-functionality-and-testing-instructions/01-architecture-spec.md`  
**Task Slug**: `button-functionality-and-testing-instructions`  
**Status**: APPROVED  
**Date**: 2026-10-20  

---

## 1. Executive Summary & Root Cause Overview

### 1.1 Problem Statement
1. **Send Button & Enqueue Button Failure**: In the Prompt Tree View, clicking "Send" or "Enqueue" (especially with image attachments or during active running sessions) fails or misbehaves. The payload structure, active instance resolution, or image forwarding mechanism is either missing images or rejecting execution.
2. **Image Clipboard Copy Inconsistency**: Copying images does not copy pure binary PNG data to the system clipboard, causing paste into desktop apps (Microsoft OneNote, Google Antigravity, Slack) to fail. The button text was labeled confusingly ("Copy All (Include Images)") instead of "Copy", and there is no batch copy mechanism for multiple images at once. Furthermore, rich markdown clipboard text should support base64 embedded data or file system path references.
3. **Prompt Detail Header Layout**: The header row displays redundant badges (`Turn T40`, `Turn 40 of 40`, `IDLE 2h`) directly preceding the session title (`Instance Management And UI Fixes`), cluttering the line. The user requested cutting off anything before the title and enlarging the title font.
4. **Tree Reset Button Deficiency**: The current Reset button only invalidates React query state or light in-memory cache without clearing the underlying SQLite database (`repo_prompts.db`), preventing users from fixing corrupt or stale transcripts. Reset must truncate/drop and rebuild the database from disk transcripts.
5. **Fleet SSH Node Git Link / Account Issue**: In the Fleet Nodes view, node accounts (especially lead / main machine vs w1) are misattributed or show `Unassigned` because GitMap SSH node configuration (`~/.gitmap/gitmap-ssh-nodes.json`) has divergent git links or identity mappings.

---

## 2. Architectural Components & Boundaries

### 2.1 Send & Enqueue Engine
- **Files**:
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/services/instanceService/lifecycle.ts`
  - `src-tauri/src/commands/cmd_running_up_next.rs`
- **Invariants**:
  - `Send` must trigger immediate execution: if the session is currently active or idle, it dispatches the prompt turn with all image attachments attached as base64 or file paths to the running Antigravity instance.
  - `Enqueue` must queue the turn into the instance's prompt queue table (`active_prompts` in SQLite), preserving image attachments and ordering (`ORDER BY created_at ASC, id ASC`).

### 2.2 Clipboard & Multi-Image Transcoding
- **Files**:
  - `src/components/instances/PromptTreeView/clipboard.ts`
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- **Invariants**:
  - Direct image copy button MUST write standard `image/png` Blob to `navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])`.
  - Transcoding to canvas ensures all image formats (JPEG, WebP, PNG) are standardized to PNG blobs.
  - Multi-image copy must support copying all images at once (either as binary blobs where supported, base64 data URIs, or local file links).
  - Button label for single image must be "Copy" (not "Copy including image").

### 2.3 Header Redesign
- **Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/TurnDetailPane.tsx`
- **Invariants**:
  - Remove turn badges preceding the title.
  - Enlarge the title text (`text-xl font-bold tracking-tight text-white`).
  - Keep the copy title icon beside the title.

### 2.4 Prompt Tree Reset & Database Regeneration
- **Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/usePromptTree.ts`
  - `src-tauri/src/commands/cmd_repo_db.rs`
  - `src-tauri/src/modules/repo_db/mod.rs`
- **Invariants**:
  - Reset action MUST invoke Tauri command `reset_prompt_tree_db(instance_id)`.
  - Backend deletes SQLite records for the instance, rescans `transcript_full.jsonl` files from disk, and repopulates the tree.

### 2.5 Fleet Nodes SSH Account & Git Link
- **Files**:
  - `src-tauri/src/modules/ssh_manager.rs`
  - `src/components/fleet/FleetNodesTable.tsx`
- **Invariants**:
  - Query GitMap SSH configuration and local machine identity accurately.
  - Ensure main node and w1 node display correct git links and assigned accounts.

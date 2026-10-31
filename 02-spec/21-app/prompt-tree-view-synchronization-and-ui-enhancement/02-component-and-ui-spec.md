# Component and UI Specification: Prompt Tree View Synchronization and UI Enhancement

## 1. Frontend Component Structure

### 1.1 `PromptTreeHeader.tsx`
- **Version Pill Badge**: Displays current version (e.g. `v4.238.0`) in a subtle dark-glass pill `px-2 py-0.5 text-[11px] rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20`.
- **Reset Button**:
  - Re-styled as a distinct segmented pill action with spinning icon during reload.
  - On click, triggers `resetPromptTreeCache()` which invokes backend cache purge and refreshes tree state.
- **Refresh & Auto-sync Interval**:
  - Encapsulated into a segmented pill capsule (`rounded-full`, dark glass).

### 1.2 `PromptTreeItem.tsx` / `ConversationItem.tsx`
- **Relative Recency Dot & Badge**:
  - In each conversation item, render a right-aligned recency tag:
    - `now`: emerald pulse dot + `now` text.
    - `30m`: purple dot + `30m` text.
    - `6h`: purple dot + `6h` text.
    - `12h`: purple dot + `12h` text.
    - `1d`: zinc dot + `1d` text.
  - Tooltip: `Last run: 2026-10-19 22:19:04`.
- **Hierarchy Correction**:
  - Verify `parent_conversation_id`: only render as a nested child if `parent_conversation_id` matches an open parent conversation in the current view.
  - Otherwise, render as a top-level sibling conversation under the workspace project folder.

### 1.3 `DetailHeader.tsx`
- **Duration & Turn Summary**:
  - Fix duration string generation: `start_time` and `end_time` must be computed from valid timestamps.
  - Time range display: `(HH:mm -> HH:mm)` with correct span (e.g. `18:40 -> 22:19` displays `3h 39m`).
  - Add explicit `Last Active` field with relative and formatted local time.

### 1.4 Dark-Glass UI Tokens & Styling Guidelines
- **Modal and Containers**:
  - Background: `bg-zinc-950/80 backdrop-blur-xl border border-white/10`.
  - Shadow: `shadow-2xl shadow-black/80`.
- **Segmented Capsules**:
  - Use `flex items-center rounded-full border border-white/10 bg-zinc-900/60 p-0.5 divide-x divide-white/5`.
  - Buttons inside: `px-3 py-1 text-xs rounded-full hover:bg-white/10 transition-colors`.

## 2. Backend Component Structure

### 2.1 `src-tauri/src/modules/repo_db/transcript.rs`
- In `compute_conversation_duration` or `extract_timestamps`:
  - Calculate `duration_ms = end_time.saturating_sub(start_time)`.
  - Guard against negative duration or incorrect hour offsets.
  - Expose `last_run_timestamp: Option<i64>` and `last_run_relative: String`.

### 2.2 `src-tauri/src/modules/repo_db/discovery.rs`
- In `scan_workspace_conversations`:
  - Scan all conversation workspaces (`conversation_summaries.db` and active transcripts).
  - Index conversations across ALL connected workspaces (including `coding-guidelines`, `gitmap`, `Antigravity-Manager`), ensuring completed conversations from hours or days ago are retained in history.

### 2.3 `src-tauri/src/commands/cmd_prompts.rs` & `server/admin_prompts.rs`
- Expose `cmd_reset_prompt_tree` / `/api/prompts/reset`:
  - Drops `prompt_tree_cache` table rows.
  - Resets memory cache timestamps.
  - Returns `{"ok": true, "purged": true, "version": APP_VERSION}`.

## 3. Telegram Inbound Media & Image Parity

### 3.1 Telegram UI Card Formatting
- `/prompts` command returns an aesthetic Markdown/HTML card listing the latest conversations with status badges, project name, turns, and relative time.
- Inline keyboard buttons:
  - `[🔍 Inspect #213]`
  - `[🖼️ View Images]`
  - `[📋 Copy Prompt]`
  - `[🔄 Re-run]`

### 3.2 Telegram Prompt Image Attachment Handling
- When a prompt contains image attachments (stored as local paths or base64 data URLs), the `/prompts image <id>` or inline button sends the actual image file to the Telegram chat using `sendPhoto` / `sendDocument`.
- Allows copying image payload directly back to the clipboard or saving to local disk.

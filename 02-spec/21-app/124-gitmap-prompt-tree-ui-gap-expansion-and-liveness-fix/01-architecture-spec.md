# Architecture Spec: 124-gitmap-prompt-tree-ui-gap-expansion-and-liveness-fix

## 1. Executive Summary & Problem Scope

This specification addresses six critical usability, display, and backend state bugs in the Antigravity Manager GitMap Prompt Tree View and instance monitoring subsystem:
1. **Vertical Line Gaps via `<br />` Tags**: Prompt instructions in both Raw and Markdown Preview modes collapse sentences together without proper paragraph line gaps. Enforce vertical line breaks using explicit `<br />` tags and structured Markdown paragraph rendering.
2. **"..." (Dot Dot) Click-to-Expand**: Truncated prompt previews ending in ellipses (`...`) do not expand when clicked. Wire interactive click-to-expand behavior on the ellipsis badge and provide an explicit toggle to expand full text.
3. **N Key Shortcut & Live "Send Now" Dispatch**: Keyboard shortcut `N` (and `n`) fails to dispatch prompts. The "Send Now" button does not reliably trigger IDE resumption. Wire global modal keydown handling for `N` and robust resume task dispatch with live visual feedback.
4. **False Positive Project Liveness (`white-presentation-v1`)**: Projects such as `white-presentation-v1` were incorrectly flagged as `RUNNING` under the Default instance even when only Antigravity was open. Root cause: lack of strict recency gates (`now - last_detected_at <= 120s`) and loose prefix matching against historical database records.
5. **Ghost "Untitled Conversation" (0 Words) Filtering & Merging**: The prompt tree was polluted with hundreds of empty conversation entries showing 0 words and "No prompt content recorded". Filter these ghost nodes out of the primary tree and consolidate empty sessions.
6. **Prompt Header Metadata Trio & Tail Excerpt**: The prompt view header lacked critical contextual indicators. Add:
   - Prompt sequence badge (e.g. `#P001` / `C001`)
   - Prompt ending text snippet (tail excerpt showing the concluding words)
   - Instance identity trio: Sequence number (`#1`), Executable name (`Antigravity.exe` / `Antigravity`), and Instance name (`default` / custom profile name).

---

## 2. Root Cause Analysis (RCA)

### 2.1 Missing Line Gaps in Text Rendering
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (`RichMarkdownRenderer` & Raw View)
- **Root Cause**:
  `RichMarkdownRenderer` rendered paragraphs using simple `<p className="my-1 ...">` and stripped whitespace. In Raw View, the text was passed as a single string inside `whitespace-pre-wrap`, but single newlines in SQLite summary text did not create visual line gaps.
- **Remediation**:
  - In `RichMarkdownRenderer`, render explicit `<br />` elements between paragraphs and lines, creating clean, unmistakable visual line breaks.
  - In Raw View, split `activePromptText` by `\n` and render line fragments separated by explicit `<br />` tags.

### 2.2 Inactive "..." Ellipsis Click-to-Expand
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (lines 1970–2005)
- **Root Cause**:
  When `activePromptText` exceeded 200/300 words, it appended `...`. The ellipsis was rendered as static text without an `onClick` handler.
- **Remediation**:
  Render the ellipsis as an interactive clickable button badge: `<span onClick={() => setShowAllWords(!showAllWords)} title="Click to expand full prompt text">... [Expand]</span>`.

### 2.3 Non-Functional 'N' Key Hotkey and Send Now
- **Location**: `src/components/instances/PromptTreeViewModal.tsx`
- **Root Cause**:
  No `window.addEventListener('keydown', ...)` was registered for the modal. When users pressed `N`, the event was ignored.
- **Remediation**:
  Register a modal-level `keydown` listener that checks for `key.toLowerCase() === 'n'` (ignoring inputs/textareas) and invokes `handleResendPrompt()`. Ensure `handleResendPrompt` writes `.antigravity_resume_task.json` and invokes `resume_recent_project_prompts` with user-facing toast feedback.

### 2.4 False Positive Liveness (`white-presentation-v1`)
- **Location**: `src-tauri/src/modules/repo_db.rs` (lines 2480–2530 & 4130–4155)
- **Root Cause**:
  `compute_project_conversation_tree` and `get_project_execution_status` inspected `active_conv_prefixes` and `conversation_summaries.db` without verifying that the project had active OS process liveness or was modified within the last 120 seconds. If an instance was alive (`is_antigravity_running == true`), old projects with stale `not_fully_idle > 0` were falsely reported as `RUNNING`.
- **Remediation**:
  Enforce a hard recency cutoff (`now - last_detected_at <= 120` seconds) and verify active worker PID liveness before declaring any project or conversation `RUNNING`.

### 2.5 Ghost "Untitled Conversation" (0 Words) Pollution
- **Location**: `src-tauri/src/modules/repo_db.rs` & `src/components/instances/PromptTreeViewModal.tsx`
- **Root Cause**:
  Empty conversation entries created by IDE initialization had no prompt content (`prompt_preview_200w.is_empty()`) and were titled "Untitled Conversation", but were still emitted in `compute_project_conversation_tree`.
- **Remediation**:
  Filter out conversation nodes where `prompt_word_count == 0` and title is untitled or empty from the primary tree in `repo_db.rs` and frontend.

### 2.6 Missing Prompt Sequence, Tail Excerpt, and Instance Trio
- **Location**: `src-tauri/src/modules/repo_db.rs` & `src/components/instances/PromptTreeViewModal.tsx`
- **Root Cause**:
  `AgmProjectTreeNode` and `AgmConversationNode` lacked `instance_exe_name`. The prompt view header only displayed the conversation title and word count.
- **Remediation**:
  Add `instance_exe_name` to `AgmProjectTreeNode` and `AgmConversationNode`. In the prompt header, render:
  - Sequence badge (`#P001` / `C001`)
  - Instance Trio: `[#1 · Antigravity.exe · default]`
  - Tail excerpt snippet: `“… ending with: <snippet>”`

---

## 3. Data Structures & Schema Updates

```rust
// In src-tauri/src/modules/repo_db.rs

pub struct AgmConversationNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub conversation_id: String,
    pub short_id: String,
    pub title: String,
    pub status: String,
    pub is_running: bool,
    pub step_count: usize,
    pub instance_id: String,
    pub instance_seq_num: Option<u32>,
    pub instance_name: String,
    pub instance_exe_name: String,
    pub prompt_preview_200w: String,
    pub prompt_tail_snippet: String,
    pub prompt_word_count: usize,
    pub last_modified: String,
}

pub struct AgmProjectTreeNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub instance_seq_num: Option<u32>,
    pub instance_name: String,
    pub instance_exe_name: String,
    pub bound_email: Option<String>,
    pub is_running: bool,
    pub conversations: Vec<AgmConversationNode>,
}
```

---

## 4. Verification & Testing Matrix

| Requirement | Test Scenario | Acceptance Criteria |
|---|---|---|
| Line Gap (`<br />`) | Open prompt with multiple paragraphs in Preview and Raw modes | Clear vertical spacing between lines/paragraphs; `<br />` tags present in DOM |
| "..." Click to Expand | Click on the trailing `...` in truncated prompt preview | Expands full prompt instruction text instantly |
| N Key & Send Now | Press 'N' hotkey or click "Send Now" button | Fires resume dispatch, writes `.antigravity_resume_task.json`, shows toast notification |
| Liveness Scoping | Open default instance with only Antigravity running | `white-presentation-v1` shows IDLE; only active projects show RUNNING |
| Untitled Convo Filter | Open project tree with numerous empty sessions | 0-word untitled conversations are filtered out from tree |
| Header Metadata Trio | Inspect prompt instruction header | Shows `#P001`, tail snippet, and instance trio `[#seq · exe · instance]` |

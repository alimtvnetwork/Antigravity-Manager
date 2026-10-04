# Architecture Spec: 126-prompt-tree-linegap-expansion-and-instance-running-fix

**Version:** 1.0.0
**Updated:** 2026-10-05
**AI Confidence:** High
**Ambiguity:** None

---

## Keywords

`prompt-tree-view` · `markdown-linegap-br` · `dot-dot-expand` · `hotkey-n-send-now` · `untitled-conversation-merge` · `instance-identity-trio` · `default-instance-liveness` · `repo-db`

---

## Scoring

| Criterion | Status |
|---|---|
| AI Confidence assigned | ✅ |
| Ambiguity assigned | ✅ |
| Keywords present | ✅ |
| Verbatim requirements captured | ✅ |
| Component hierarchy & data flow defined | ✅ |
| Root Cause Analysis (RCA) detailed | ✅ |
| Non-negotiable constraints documented | ✅ |

---

## 1. Executive Summary & Verbatim User Requirements

This specification formalizes the architectural design, component hierarchy, data flow, and verification gates for task **126-prompt-tree-linegap-expansion-and-instance-running-fix** in Antigravity Manager.

### 1.1 Verbatim Requirements
1. **Line gap fix in UI text display using BR tags only, formatted for markdown**: Prompt instructions in both Rich Markdown Preview and Raw Monospace modes must preserve explicit line breaks and vertical paragraph gaps using explicit `<br />` tags rather than collapsing into single dense text blocks or relying on CSS margins alone.
2. **Dot-dot `...` click-to-expand to full text**: Truncated prompt previews ending in ellipses (`...` or `…`) must be interactive, clickable badges that instantly toggle and expand the full prompt text in place.
3. **`N` hotkey and "Send Now" button live functionality**: Pressing the `N` key (or `n`) in the modal or clicking the "Send Now" button must immediately dispatch the prompt to the running instance by writing `.antigravity_resume_task.json`, triggering IDE resume IPC, copying prompt text to the clipboard, and focusing the active IDE window.
4. **Filter out & merge untitled conversations with no prompt content**: The prompt tree must purge 0-word untitled ghost sessions and consolidate empty or stale sessions into an "Archived / Stale Prompts" collapsible group.
5. **Prompt view header displaying prompt sequence, ending text snippet, and instance ID trio**: The prompt instruction panel header must render:
   - Prompt sequence badge (e.g., `#P001` or `C001`).
   - Ending text snippet (e.g., `“… ending with: 'concluding text'”`).
   - Instance identity trio: `[#Id sequence · Id exe name · Id instance name]` (e.g., `[#1 · Antigravity.exe · default]`).
6. **Root cause and fix for incorrect project activity under Default instance**: Resolve false-positive `RUNNING` status where idle projects (e.g. `white-presentation-v1`) were falsely reported as active under the Default instance when only the host IDE process was running.

---

## 2. Root Cause Analysis (RCA)

### 2.1 UI Line Collapse in Markdown Preview and Raw Views
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (`formatPromptForMarkdown`, `RichMarkdownRenderer`, and Raw View tab).
- **Underlying Cause**:
  1. In `formatPromptForMarkdown`, only specific heading patterns were spaced. Regular single linebreaks and double linebreaks in raw database summary text were parsed by standard React JSX paragraph mapping without injecting explicit break elements.
  2. In `RichMarkdownRenderer`, paragraphs were rendered inside `<p>` blocks. Browsers and markdown tokenizers collapse adjacent inline text and whitespace.
  3. In Raw View, passing a single string inside `whitespace-pre-wrap` did not enforce deterministic vertical separation when text contained single `\n` characters from SQLite summaries.
- **Resolution**:
  - `formatPromptForMarkdown` standardizes paragraph boundaries with explicit spacing.
  - `RichMarkdownRenderer` injects `<br className="my-1 block select-none" />` elements after each paragraph wrapper and `<br className="my-2" />` for blank lines.
  - Raw View splits `activePromptText` by `\n` and renders each line with an explicit `<br className="my-1 block select-none" />` separator.

### 2.2 Inactive "..." Ellipsis Click-to-Expand
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (Preview Mode & Ellipsis Badge).
- **Underlying Cause**:
  Truncated text was sliced at 120 words with a trailing `...` string appended as non-interactive text. Users had to locate distant toolbar buttons to expand the text.
- **Resolution**:
  Transform the trailing ellipsis into an interactive clickable badge with `onClick={() => setShowAllWords(!showAllWords)}`, distinct styling (`bg-cyan-500/10 text-cyan-600 hover:underline`), and descriptive tooltip (`"Click to expand full prompt text"`).

### 2.3 Non-Functional `N` Key Hotkey and Send Now Dispatch
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (`handleResendPrompt`, window keydown event listener).
- **Underlying Cause**:
  The modal did not maintain an active window keydown listener for the `N` key. Furthermore, "Send Now" previously lacked guaranteed file-based resume dispatch (`.antigravity_resume_task.json`) and window focusing (`focusOrLaunchInstance`), causing prompt execution to fail silently if IPC was disconnected.
- **Resolution**:
  1. Register a modal-scoped `window.addEventListener('keydown', ...)` checking for `e.key === 'n' || e.key === 'N'` (ignoring inputs, textareas, selects, and contenteditable elements).
  2. In `handleResendPrompt`:
     - Write `.antigravity_resume_task.json` into the project root path.
     - Call `invoke('resume_recent_project_prompts')`.
     - Write prompt text to the clipboard.
     - Call `focusOrLaunchInstance(targetInstId)` to bring the IDE instance window to the foreground.
     - Display action toast feedback.

### 2.4 False-Positive Project Activity Under Default Instance
- **Location**: `src-tauri/src/modules/repo_db.rs` (`is_prompt_running_for_project`, `compute_project_conversation_tree`, `get_live_project_execution_info`).
- **Underlying Cause**:
  1. When Antigravity was open, `is_antigravity_running(None)` returned `true`.
  2. `is_prompt_running_for_project` inspected historical rows in `conversation_summaries.db` and `active_prompts`. If a project had historical turns with `not_fully_idle > 0` or loose prefix matching, it was marked `RUNNING` regardless of turn age.
  3. In `compute_project_conversation_tree`, if a project had empty workspace folders on disk, fallback checks inspected global active prompt tables without strict TTL constraints.
- **Resolution**:
  1. Enforce a strict 60-120s TTL cutoff (`now - conv_ts <= 60` or `<= 120`).
  2. Enforce strict idle supremacy: if `not_fully_idle == 0` or status contains `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, the session is unconditionally marked `IDLE`.
  3. Validate path existence and verify that active conversation prefixes match decodable workspace paths (`clean_p == clean_target`).
  4. Ensure only projects with live, active worker PIDs or verified in-flight turns within the TTL window are marked `RUNNING`.

### 2.5 Ghost "Untitled Conversation" (0 Words) Pollution
- **Location**: `src-tauri/src/modules/repo_db.rs` & `src/components/instances/PromptTreeViewModal.tsx`.
- **Underlying Cause**:
  New empty sessions generated during IDE initialization created SQLite records with `title = "Untitled Conversation"` and `prompt_preview_200w = ""`. These were rendered as clutter in the project tree.
- **Resolution**:
  1. Filter out ghost conversations where `prompt_word_count == 0` and title is untitled/empty in both backend (`repo_db.rs`) and frontend (`PromptTreeViewModal.tsx`).
  2. Stale or non-ghost empty conversations are segregated into a collapsible `Archived / Stale Prompts (N)` group to keep the primary tree clean.

### 2.6 Missing Prompt Sequence, Tail Excerpt, and Instance Trio
- **Location**: `src-tauri/src/modules/repo_db.rs` & `src/components/instances/PromptTreeViewModal.tsx`.
- **Underlying Cause**:
  Header only displayed conversation title and word count, omitting essential instance context and prompt tail snippets.
- **Resolution**:
  1. Populate `seq_code` (e.g. `#P001` or `C001`), `prompt_tail_snippet` (concluding 12 words), and instance identity fields (`instance_seq_num`, `instance_exe_name`, `instance_name`).
  2. Render the instance identity trio `[#seq · exe · instance]` and tail excerpt snippet in the header and preview toolbar.

---

## 3. System Architecture & Component Hierarchy

```
+-----------------------------------------------------------------------------------+
|                            PromptTreeViewModal.tsx                                |
+-----------------------------------------------------------------------------------+
  |
  +--> [Toolbar & Global Controls]
  |      |-- Search Input & Project Scope Filter
  |      |-- Filter Pills: All / Running / Latest Conv / Pinned / Archived
  |      |-- Hotkey Listener: Window KeyDown ('N' / 'n' -> Send Now)
  |
  +--> [Split Body: 2-Column Responsive Layout]
         |
         +--> [Left Column: Project & Conversation Tree]
         |      |-- Project Node (renderProjectNode): Folder icon, Pin, Archive, Prompt Count
         |      |-- Active Conversations: Filtered (isGhostConversation dropped)
         |      |-- Stale Group Accordion: "Archived / Stale Prompts (N)"
         |
         +--> [Right Column: Prompt Detail & Actions Panel]
                |
                +--> [Header Context Bar]
                |      |-- Sequence Badge: #P001 / C001
                |      |-- Conversation Title & Running Status Badge (PID + Duration)
                |      |-- Project Folder Badge
                |      |-- Instance Trio: [#Id seq · Id exe · Id name]
                |      |-- Tail Excerpt: "… ending with: '<snippet>'"
                |
                +--> [Action Button Capsule]
                |      |-- Copy Text (clean) / Copy With Images (verbatim) / Save Images
                |      |-- Suffix Dropdown ("Is it done?", "Verify build", etc.)
                |      |-- Focus IDE (ExternalLink)
                |      |-- Send Now (RotateCw + [N] Hotkey Badge) -> handleResendPrompt
                |      |-- Enqueue (FIFO scheduler) / Full Screen Inspector
                |
                +--> [View Mode Tabs: Preview | Raw | Edit]
                       |
                       +--> Preview Tab: RichMarkdownRenderer
                       |      |-- Pre-formatting: formatPromptForMarkdown
                       |      |-- Paragraphs & Headings with explicit <br /> tags
                       |      |-- Clickable Ellipsis Badge: "... [Expand Full Text]"
                       |
                       +--> Raw Tab: Monospace Clean Text
                       |      |-- Line-by-line mapping with explicit <br /> tags
                       |
                       +--> Edit Tab: Textarea Modification
```

---

## 4. Data Flow & Execution Pipelines

### 4.1 UI Line Gap Rendering Pipeline (`<br />` Only)

```
[Raw SQLite Summary / Active Prompt Text]
                  │
                  ▼
      [formatPromptForMarkdown(text)]
         - Normalizes headings (#, ##, ###)
         - Preserves paragraph double linebreaks
                  │
                  ▼
         [RichMarkdownRenderer]
         - Splits by '\n' into lines
         - For empty line: <br className="my-2" />
         - For paragraph:
             <div className="my-1.5 leading-relaxed">
                 <p className="...">{parseInlineMarkdown(line)}</p>
                 <br className="my-1 block select-none" />
             </div>
                  │
                  ▼
    [DOM Output: Guaranteed Vertical Separation Without Collapse]
```

### 4.2 Ellipsis Expansion Pipeline

```
[Prompt Text Length > 120 words]
                  │
                  ▼
    [previewDisplayText with '...']
                  │
                  ▼
      [showAllWords State (boolean)]
         ├── false: Render previewDisplayText + Clickable Badge
         └── true:  Render activePromptText + Collapse Badge
                  │
                  ▼
   [User Clicks '... [Expand Full Text]']
                  │
                  ▼
    [setShowAllWords(true) -> Instant Full-Text DOM Re-render]
```

### 4.3 "Send Now" & Hotkey `N` Execution Pipeline

```
[User Presses 'N' Hotkey OR Clicks "Send Now" Button]
                  │
                  ▼
     [Target Input Guard Check]
     - If active element is <input>, <textarea>, or contenteditable: IGNORE
                  │
                  ▼
        [handleResendPrompt()]
         1. Resolve selected conversation & project repo_path
         2. Append confirmation suffix (if selected)
         3. Write .antigravity_resume_task.json to project root
         4. invoke('resume_recent_project_prompts', { instanceId, maxAgeSeconds: 3600 })
         5. navigator.clipboard.writeText(promptContent)
         6. focusOrLaunchInstance(targetInstId)
         7. setActionMsg("Prompt Dispatched & Focused IDE (via Hotkey 'N' / Send Now)!")
```

### 4.4 Liveness & Recency Gate Verification Pipeline (Backend)

```
[Project Liveness Probe: is_prompt_running_for_project]
                  │
                  ▼
  [Gate 0: Host Process Liveness]
  - Is Antigravity running for target instance?
  - NO -> Return false (INSTANCE_PROCESS_DEAD)
                  │
                  ▼
  [Gate 1: In-Memory Prompts Map]
  - Status == 'running' AND (now - updated_at) <= 120s?
  - YES -> Return true (ACTIVE_PROMPT_MEMORY)
                  │
                  ▼
  [Gate 2: Active AGY Workers Map]
  - Scoped by instance_id, matching path, live OS PID?
  - YES -> Return true (ACTIVE_WORKER_MATCHED)
                  │
                  ▼
  [Gate 3: SQLite active_prompts Table]
  - Status == 'running' AND updated_at >= now - 120s?
  - YES -> Return true (ACTIVE_PROMPT_DB_RUNNING)
                  │
                  ▼
  [Gate 4: Live conversation_summaries.db]
  - not_fully_idle > 0 AND status == 'RUNNING'
  - Recency TTL: (now - conv_time) <= 120s
  - Decodable workspace URI == project path?
  - YES -> Return true (CONVERSATION_SUMMARY_ACTIVE_TURN)
                  │
                  ▼
  [All Gates Fail -> Return false (IDLE_NO_ACTIVE_TASKS)]
```

---

## 5. Non-Negotiable Constraints

1. **BR Tags Only for Line Gaps**: Text formatting in markdown preview and raw views must inject explicit `<br />` tags to guarantee vertical spacing across different operating systems and webview rendering engines. Do not rely solely on CSS margin collapsing.
2. **Zero Git Commands**: Development and specification workflows must strictly avoid executing git commands (`git add`, `git commit`, `git status`, `git diff`, `git checkout`).
3. **Ghost Conversation Purging**: Zero-word untitled conversations must never be displayed in the primary active conversation list.
4. **Strict Liveness Recency Gate**: A project must NEVER be declared `RUNNING` unless verified against an active OS process PID and a live conversation turn updated within 120 seconds.
5. **Hotkey Isolation**: The `N` hotkey listener must be active only when the modal is open, and must never intercept typing in input fields, textareas, or the 'Edit' tab textarea.
6. **Cross-Platform Compatibility**: Paths, process matching, and file operations must operate seamlessly across Windows, macOS, and Linux without platform-specific hardcoding.

---

## 6. Verification & Acceptance Criteria

| Requirement | Test Scenario | Expected Outcome |
|---|---|---|
| **Line Gap `<br />` Tags** | Open multi-line prompt instruction in Preview and Raw views | Explicit `<br />` tags present in DOM; clear vertical spacing between all paragraphs |
| **"..." Click to Expand** | Click on trailing `...` in truncated prompt | Instantly expands full prompt instruction in place without modal jitter |
| **Hotkey `N` / Send Now** | Press `N` key or click "Send Now" button | Emits `.antigravity_resume_task.json`, triggers resume IPC, copies text, focuses IDE window |
| **Default Instance Liveness** | Launch Default instance with idle projects (e.g. `white-presentation-v1`) | Project displays `IDLE`; only projects with active turns within 120s display `RUNNING` |
| **Ghost Convo Filtering** | Inspect project with 0-word empty sessions | Ghost nodes are purged; stale sessions grouped under "Archived / Stale Prompts" |
| **Header Identity Trio** | Inspect prompt view header | Header displays `#P001`, `[#1 · Antigravity.exe · default]`, and tail excerpt snippet |

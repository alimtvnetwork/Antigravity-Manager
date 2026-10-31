# Master Plan: Prompt Tree Subtree Hierarchy, Accurate Running State, Section Import/Export with Base64 Images, and Typography Overhaul

> **Task Slug**: `prompt-tree-subtree-and-import-export`  
> **Status**: `ACTIVE (Phase 2 Implementation)`  
> **Target Version**: `v4.222.0`  
> **Concurrency**: `A = 3, H = 2` (3 concurrent worker subagents)  
> **SQLite Task Ledger**: `.ai-memory/temp-agents/12-prompt-tree-subtree-and-import-export/agent-task.db`  

---

## User Request (Verbatim)

```text
One more very serious issue in your prompt tree view that usually how it should be is that you have the project name. Inside this, you should have the conversation name, and inside this it would have the prompts sending list. I think this is where you are messing up. And as you can see in my screenshot, I have two different screenshot I have provided. One is your prompt view and another is the actual IDE view. In the IDE it is continuing previous task. That is fine. The name matches. But you don't have a subtree of the prompts that I have sent. That is the problem. And it only shows the last prompt that has been sent, but not the running symbol. It does not have a running symbol which prompts are running at the moment, which are enqueued. You can see there a lot of prompts are in queue which is not showing up here. That is one of the big issues, I think you need to find the root cause by running the code into the machine and make sure that you get the result. At the end it's a very simple task. First write the root cause, find the root cause, and then try to fix it and make sure you write the root cause analysis so that this mistake never happens. And make sure that every one of the prompt tree we could right click and export the full tree item for that section. Import export as a... There should be import export button. If we go into that import export, that would have... You can export all of them together as a JSON or database. And we could also import in a different machine. Make sure this import export works for everything. If there are images, convert that to base64 and include that into the prompt or the database. And also when importing, convert that back to images and put it into the attachment or specific place. That would be a good option. Apparently, the refresh button also does not work. It feels a little bit buggy. Can you look into the refresh button, why it is failing or why it does not show the result immediately? And the font for all the text should be Poppins. For the title, it should be Ubuntu. Please try to follow this. That would look much better. And this conversation should also have a copy button at the top header so that we can copy the conversation. And also beside the conversation, we should have the prompt resend option, like resend last 5, resend last 10, or resend all. And also, verify the last 10 prompts. That would verify that everything is being verified or not. And clone conversation, which would take that and run the compact memory, and then send it to the prompt. What does C218, C17 mean? Please let me know so that I can understand.
```

---

## Architectural Taxonomy & RCA Summary

1. **RCA - False IDLE Status & Missing Running Symbol (Screenshot 02 vs 03)**:
   - **Flaw 1 (Terminal Done Short-Circuit)**: In `src-tauri/src/modules/repo_db/transcript.rs` (lines 484-535), `inspect_conversation_transcript` reverses through tail lines. When encountering `PLANNER_RESPONSE` or `source: 'MODEL'` turn with `has_tool_calls == false`, it set `is_terminal_done = true`. In Antigravity IDE, parent agents emit intermediate planner turns without tool calls before spawning workers or subagents!
   - **Flaw 2 (Override of `not_fully_idle`)**: In `tree_collect.rs` (lines 169-177), `is_conv_running` evaluated `if is_explicit_idle || inspection.is_terminal_done { false }`. This short-circuited and ignored `not_fully_idle > 0` in `conversation_summaries.db`! Even though the IDE was actively running and subagents were executing (`CASCADE_RUN_STATUS_RUNNING`), `is_terminal_done` forced `is_conv_running = false`.
   - **Flaw 3 (Premature IDLE Mutation)**: In `tree_node_conv.rs` (lines 112-120), because `effective_is_run` was false, line 116 matched `item.status.contains("RUNNING")` and forcibly mutated the active conversation into `"IDLE"`.
   - **Fix**: In `tree_collect.rs`, runtime process liveness + database un-idle counter strictly overrides static transcript heuristics: `if is_owning_inst_alive && not_fully_idle > 0 && is_recent => is_conv_running = true`.

2. **3-Tier Hierarchy Data Model (`Project` → `Conversation` → `Prompt Subtree`)**:
   - Add `AgmPromptTurnNode` (`turn_number`, `step_index`, `prompt_text`, `preview_200w`, `status` ['RUNNING' | 'QUEUED' | 'COMPLETED'], `is_running`, `is_queued`, `timestamp`, `created_at`, `media_paths`, `media_thumbnails`) to `AgmConversationNode.prompts: Vec<AgmPromptTurnNode>`.
   - In `transcript.rs`, parse all `USER_INPUT` turns from `transcript_full.jsonl` (and `media` array pointing to `brain/<cid>/.user_uploaded/media_*.png` or `.antigravity/attachments/`).
   - In `ConversationNode.tsx`, render an expandable prompt subtree beneath the conversation (`Turn 1, Turn 2, ..., Turn N`), with pulsing emerald badge for RUNNING, amber for QUEUED, zinc/gray for COMPLETED.

3. **Section-Level Import / Export (JSON, SQLite DB, YAML) with Base64 Images**:
   - Right-click context menus on tree nodes (`ProjectContextMenu.tsx`, `ConversationContextMenu.tsx`, `PromptNodeContextMenu.tsx`): Export Section as JSON, SQLite `.db`, or YAML.
   - Top toolbar Import / Export modal (`PromptTreeExportModal.tsx`, `PromptTreeImportModal.tsx`).
   - **Base64 Image Embedding Invariant**: Read all referenced images/attachments from disk, encode as Base64 data URIs (`data:<mime>;base64,...`), and store in `embedded_images: HashMap<String, String>` (or SQLite `embedded_images` table). On import, decode and materialize back to `<target>/.antigravity/attachments/` with zero broken links.

4. **Refresh Button 4-Point RCA & Fix**:
   - **Selection Whiplash**: `onRefreshAll` called `loadTree(true, true)` with `isInitialLoad: true`, which triggered `performAutoSelection()` and forcefully wiped user selection. Fix: call `loadTree(false, true)` preserving selection.
   - **Stale Workspace Cache**: call `detect_running_projects(target_instance)` prior to querying tree nodes.
   - **SQLite WAL Frame Bypass**: eliminate `immutable=1` fallback when opening `conversation_summaries.db`, open read-only with 5000ms `busy_timeout`.
   - **45s Cutoff**: raise cutoff in `tree_phases.rs:15` to 600s.

5. **Typography & Header Copy Button**:
   - Enforce **Ubuntu** font (`font-ubuntu`) for all headers and titles.
   - Enforce **Poppins** font (`font-poppins`) for all body, prompt text, previews, and badges.
   - Add 1-click Copy button directly beside conversation title `<h3>` in `DetailHeader.tsx` with copy toast.

6. **Sequence Code Explanation (`C218`, `C17`, `P003`)**:
   - `C` = Conversation sequential index (`agm_conversation_sequences`), `P` = Project sequential index (`agm_project_sequences`).
   - Add informative tooltips: `Conversation #226 (Sequential AGM Index)` and `Project #003 (Sequential Project Index)`.

7. **Conversation Actions on Tree**:
   - Inline double-click, F2 hotkey, and right-click rename in `ConversationContextMenu.tsx`.
   - Multi-Prompt Resend submenu: "Resend All", "Resend Last 5", "Resend Last 10", "Resend Custom N...".
   - "Verify Last 10 Prompts & Enqueue": Autonomous 5-chapter prompt verification synthesis enqueued back to IDE.
   - "Clone Conversation & Compact Memory": Session handoff generator creating a compacted summary prompt.

---

## Subtask Inventory & Ownership (A = 3, H = 2)

| Subtask ID | Task Code | Title | Assigned Agent | Owned Target Files | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `TASK-01` | Prompt Subtree Hierarchy, Turn Extraction, and False IDLE RCA Fix | Worker 01 | `src-tauri/src/modules/repo_db/models.rs`, `transcript.rs`, `tree_collect.rs`, `tree_node_conv.rs`, `tree_nodes.rs`, `tree.rs`, `src/components/instances/PromptTreeView/types.ts`, `ConversationNode.tsx`, `TreePanel.tsx` | PENDING |
| 2 | `TASK-02` | Section Import/Export (JSON/DB/YAML) with Base64 Images & Refresh RCA Fix | Worker 02 | `src-tauri/src/modules/repo_db/sqlite_export.rs`, `export.rs`, `import.rs`, `tree_phases.rs`, `src-tauri/src/commands/project_cmds.rs`, `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`, `PromptTreeExportModal.tsx`, `PromptTreeImportModal.tsx`, `ProjectContextMenu.tsx`, `PromptTreeViewModal.tsx` | PENDING |
| 3 | `TASK-03` | Poppins & Ubuntu Typography, Header Copy, Sequence Tooltips & Conversation Actions | Worker 03 | `src/components/instances/PromptTreeView/DetailHeader.tsx`, `PromptTreeDetail.tsx`, `ConversationContextMenu.tsx`, `usePromptTreePromptActions.ts`, `verificationTemplates.ts`, `index.html`, `tailwind.config.js`, `src/App.css` | PENDING |
| 4 | `TASK-04` | Conversation Renaming Hotkeys, Resend Submenus, and Compact Clone Engine | Worker 01 | `src-tauri/src/modules/repo_db/sequences.rs`, `src-tauri/src/commands/project_cmds.rs`, `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`, `ConversationNode.tsx` | PENDING |
| 5 | `TASK-05` | Pre-flight Quality Gates, Zero-CI Verification, and Minor Release Ceremony (v4.222.0) | Lead | `src-tauri/Cargo.toml`, `package.json`, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md` | PENDING |

---

## Wave Schedule
- **Wave 1 (Parallel Backend & UI Foundations)**:
  - Worker 01: `TASK-01` (Prompt Subtree Hierarchy, `AgmPromptTurnNode`, Transcript parsing, False IDLE fix)
  - Worker 02: `TASK-02` (Section Import/Export for JSON/SQLite/YAML, Base64 image embedding/materialization, Refresh button RCA fix)
  - Worker 03: `TASK-03` (Poppins/Ubuntu fonts, 1-click header copy, C-sequence tooltips, verification prompt synthesis)
- **Wave 2 (Actions Polish & Integration)**:
  - Worker 01: `TASK-04` (F2 hotkey, Resend N submenu dialogs, Clone conversation with compacted memory)
- **Wave 3 (Pre-flight Gates & Release)**:
  - Lead: `TASK-05` (Cargo fmt, clippy, build, gitmap e d, bump minor to `v4.222.0`, update changelogs/readmes strictly attributing `@aukgit`, commit, tag, push)

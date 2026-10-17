# Master Plan: Prompt Restore Labeling, Conversation Title Editing, and Prompt Tree Overhaul

> **Task Slug**: `prompt-restore-labeling-and-tree-overhaul`  
> **Status**: `ACTIVE (Phase 1 Planning & Spec)`  
> **Target Version**: `v4.222.0`  
> **Concurrency**: `A = 3, H = 2` (6 concurrent subtask operations)  
> **SQLite Task Ledger**: `.ai-memory/temp-agents/11-prompt-restore-labeling-and-tree-overhaul/agent-task.db`  

---

## User Request (Verbatim)

```text
The backup and restore prompt is working, but it is not working correctly. The issue here is that when you send the prompt 10 seconds later, we should be able to see this somewhere with labeling in the instances. It could be a label like "Waiting to restore this many of the prompts," meaning all the running prompts, or send them again. This needs to show up, and if people hover over and click, they would see the operation that's going to happen 10 seconds later. Second, you need to correctly find what was the running prompt. The running prompt section is correct. You can detect when the prompts are running, which is good. But the main problem is you get the wrong prompt. That is the main issue at the moment. When we have a conversation on the right-hand side, we should be able to double-click and change the conversation title. When we get into a conversation, we should be able to right-click and change the conversation name, and also double-clicking on the conversation on the top-level header, we should be able to change the conversation name directly. At this point, you have not merged the buttons, the copy, send all, and rerun. That should be all together. It's not done yet. The suffix should be a template library, so there should be more, and the user can also add a suffix template into the send section. Try to add that option. Everything should also have the CLI options so that the user can customize from the CLI automation. If you fix this, it would be a top-notch product. The work is good, but there are some glitches, and the markdown preview is good, but not professional enough. It does not feel very charming. Look into how to make it a more charming UI/UX concept. Keep the prompt and results section on the right-hand side or together with the preview section. In the preview section, you can have this icon. Write this prompt instruction as input and output. Put it into the section where highlighted. Verify the code or create a verification prompt so that AI can verify the task. At the end, do a minor bump and release the product. Before that, check the build on the machine. Do not run too many tests. Send it to the CI/CD and check the gitmap E space D. E, D. Space D, not D-E, to check for any CI/CD issues. If there is, try to minor bump until you reach CI/CD green. That would be a goal.
```

---

## Architectural Taxonomy & RCA Summary

1. **RCA - Selecting the 'Wrong Prompt' in Post-Switch Resume & Running Detection**:
   - `discovery.rs` (lines 152-193) scanned `transcript.jsonl` in reverse looking for `type == "USER_INPUT"`. Subagent prompts and tool injections (`<SYSTEM_MESSAGE>`, `[Message] timestamp=...`, `You are Research...`) are logged under `USER_INPUT`. `discovery.rs` did not use `is_subagent` or `extract_clean_user_prompt`, grabbing subagent directives instead of the user's root goal.
   - Truncated text (`<truncated N bytes>`) was injected into `active_prompts` when reading `transcript.jsonl` instead of un-truncated `transcript_full.jsonl`.
   - Fallback to turn-0 `conversation_summaries.db` preview pulled ancient conversation openers.
   - `backup.rs:488` dumped editor state from `state.vscdb` `%prompt%` / `%chat%` keys.
   - `backup.rs:580` executed an inverted LIFO query (`ORDER BY updated_at DESC LIMIT 1`).
   - `restore.rs:114` destroyed prompt creation timestamps by overwriting `created_at` with `now`.
   - `switch.rs:155` queried backup against target instance instead of active instance.

2. **Pending Post-Switch Resume 10s Labeling**:
   - Track scheduled post-switch tasks in `post_switch_resume.rs` via `ScheduledResumeEntry` (`instance_id`, `fire_at_ms`, `prompt_count`, `prompts: Vec<PendingPromptPreview>`).
   - Expose `get_pending_post_switch_resumes() -> Vec<PendingResumeTaskInfo>`, `cancel_post_switch_resume()`, `force_dispatch_pending_resume()`.
   - Render interactive amber badge `[RotateCw animate-spin] Restoring N in Xs` in `InstanceCard.tsx` and `InstanceTable.tsx` with hover preview and click modal.

3. **Conversation Title Renaming Across 3 Entry Points**:
   - Backend `rename_conversation(instance_id, conversation_id, new_title)` updating `conversation_summaries.db` (with `busy_timeout`) and `agm_conversation_sequences` in `repo_prompts.db`, then invalidating cache.
   - Entry Point 1: Inline double-click on title in `ConversationNode.tsx` with `e.stopPropagation()` (preventing inspector modal trigger).
   - Entry Point 2: Right-click context menu `ConversationContextMenu.tsx` with "Rename Conversation".
   - Entry Point 3: Double-click on top-level conversation title in `DetailHeader.tsx`.

4. **Prompt Tree Toolbar Action Merging**:
   - Merge `Copy All`, `Send N / Enqueue`, and `Rerun ×N` into a single contiguous segmented dark-glass capsule (`rounded-full`, shared border, `divide-x`, subtle dividers).

5. **Suffix Template Library**:
   - Built-in expanded presets (`is-it-done`, `is-it-released`, `are-you-confident`, `edge-cases-and-tests`, `security-audit`, `pr-review-ready`, `test-and-coverage`, `refactor-dry`).
   - SQLite table `prompt_suffix_templates` in `repo_prompts.db` for custom user templates.
   - Custom dark-glass dropdown replacing the jarring OS white `<select>` dropdown.
   - CLI verbs: `agm prompt suffix list/get/add/remove/set-default`.

6. **Charming Markdown Preview & Dual Input/Output Inspector**:
   - Split view in `InspectorModal.tsx` and `PromptTreeDetail.tsx`: Side-by-Side (wide) / Stacked (compact) with Input (Prompt Instruction) and Output (AI Results + Tools).
   - High-contrast codeblocks with language tags and copy button, collapsible reasoning `<thinking>` card.
   - Structured verification prompt generator for automated AI task validation.

---

## Subtask Inventory & Waves (A = 3, H = 2)

| Subtask ID | Task Code | Title | Assigned Agent | Target Files | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `TASK-01` | Correct Running Prompt Matching & 10s Restore Labeling | Worker 01 | `src-tauri/src/modules/repo_db.rs`, `post_switch_resume.rs`, `InstanceCard.tsx`, `InstanceTable.tsx` | PENDING |
| 2 | `TASK-02` | Conversation Title Editing (Inline Double-Click & Right-Click) | Worker 02 | `project_cmds.rs`, `tree.rs`, `ConversationNode.tsx`, `DetailHeader.tsx` | PENDING |
| 3 | `TASK-03` | Prompt Tree Toolbar Action Merging & Suffix Template Library | Worker 03 | `DetailHeader.tsx`, `verificationTemplates.ts`, `suffix_templates.rs`, `prompt_cmds.rs` | PENDING |
| 4 | `TASK-04` | Charming Markdown Preview & Integrated Input/Output Inspector | Worker 02 | `PromptMarkdownPreview.tsx`, `PromptTreeDetail.tsx`, `InspectorView.tsx` | PENDING |
| 5 | `TASK-05` | CLI Parity, Pre-flight Quality Gates, and Release Ceremony | Lead | `Cargo.toml`, `package.json`, `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md` | PENDING |

---

## Wave Schedule
- **Wave 1 (Parallel Backend & UI Foundations)**:
  - Worker 01: `TASK-01` (Correct prompt detection, 10s restore tracking & instance card badges)
  - Worker 02: `TASK-02` (Conversation title backend command & 3-point UI editing)
  - Worker 03: `TASK-03` (Toolbar action merging & suffix template library + CLI)
- **Wave 2 (UI Polish, Markdown Inspector & Verification)**:
  - Worker 02: `TASK-04` (Charming markdown preview, dual input/output inspector & AI prompt generator)
  - Lead: `TASK-05` (CLI parity, pre-flight cargo fmt/clippy/build, release ceremony to `v4.222.0`)

# Completed Plan: 128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix

## User Request (Verbatim)

```text
Okay. So here, if you look into this, the UI does not look okay. So first of all, the line gap, you don't have the line gap. How you get the text, you don't have the line gap. Fix the line gap in terms of display and everywhere. Okay? That's the first. Second, the issue here, because the line gap, you need to fix it with a BR tag only. That's the first thing. Okay. So apply that, and then format it for the markdown. Okay. Then when I click on dot dot, it does not expand to the full text. That is a problem, and also N key, or also Send Now does not work at all. So you need to test it live here, then it goes there, and you can trace back. And also the activity, like where the project is running or things like that. That is also incorrect. For example, if I give you the screenshot of the defaults, the default instance does not have anything but Integrab be running. But it says what presentation is running. This is absolutely wrong. So you have to find the root cause to see why it is happening. And if we go into the `gitmap` section. You want to get into the `gitmap` section, the Git maps prompt. If we open, the problem here is that it still has prompts which has untitled conversation and has no prompt content. I asked you not to show this in the list while you are seeing this. And also, in this UI, the prompt view, I need to see a little bit of prompts sequence, prompts ending text, and also the name of the instance of the ``Id``. So again, ``Id`` sequence, ``Id`` exe name, and also ``Id`` instance name. These three things I need to see here in the UI nicely. You need to move one prompt. You need to check the running prompts and also the running prompts property, sending the running prompts, entering the prompt. You need to check all this. It's typically working in the past, but in your case it is not working. Lots of untitled conversations. I asked you to merge these untitled conversations which has no prompt. Try to merge this. Yet understood

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Fix the line gap in the UI using a BR tag and ensure it displays correctly everywhere.
4. Format the text for markdown.
5. Resolve the issue where clicking on "dot dot" does not expand to the full text.
6. Test the N key and Send Now functionality live to ensure they work correctly.
7. Investigate and correct the activity display issue where incorrect project running information is shown.
8. Find the root cause of the incorrect default instance display.
9. In the `gitmap` section, ensure untitled conversations with no prompt content are not shown in the list.
10. Update the UI to display prompt sequence, prompt ending text, and the name of the instance of the ``Id``.
11. Check and ensure the running prompts and their properties are functioning correctly.
12. Merge untitled conversations that have no prompt content.

## Visual Reference Assets
- ![Prompt Instructions Note](assets/screenshots/128-prompt-tree-live-01.png)
- ![Accounts Quota Progress Bars](assets/screenshots/128-prompt-tree-live-02.png)
- ![Show All Quotas Header Toggle](assets/screenshots/128-prompt-tree-live-03.png)
- ![Theme Catalogue Dropdown](assets/screenshots/128-prompt-tree-live-04.png)
```

---

## 1. Executive Summary & Root Cause Synthesis

### 1.1 UI Line Gaps via `<br />` Tags Only
- **Root Cause:** Standard markdown HTML `<p>` tags with inline `<br>` elements collapse under browser margin rules.
- **Solution:** Standardized `formatPromptForMarkdown` and injected block-level select-none `<br className="my-1.5 block select-none" />` elements on all newlines in `RichMarkdownRenderer`, inline markdown parser, and Raw View tab to enforce true vertical line spacing everywhere.

### 1.2 Interactive Dot-Dot (`...`) Click-to-Expand
- **Root Cause:** Truncated text ending with `...` inside lists, headings, and code blocks was flushed before paragraph parsing, and inspector modals lacked `onToggleExpand` forwarding.
- **Solution:** Supported `isTruncated`, `showAllWords`, and `onToggleExpand` universally; bound interactive click handlers on the ellipsis token `[... [Expand Full Text]]` / `[... [Collapse]]` in preview, raw, and inspector modals.

### 1.3 `N` Key Hotkey & "Send Now" Resend Live Execution
- **Root Cause:** Missing active conversation/project resolution caused silent returns; `get_dispatched_prompts_cache` blocked previously sent signatures; missing focus calls left the IDE in background.
- **Solution:** Added automatic fallback resolution to newest non-empty conversation; writes `.antigravity_resume_task.json` using `save_text_file`; triggers `resume_recent_project_prompts`; copies prompt to clipboard; focuses target IDE window with toast feedback; verifies live file generation.

### 1.4 Stale DB Cache & False Running Activity on Default Instance (`white-presentation-v1`)
- **Root Cause:** 
  1. `prompt_tree_cache` in `repo_prompts.db` retained stale cached JSON (`tree:all:50:false`) with `is_running: true` for `white-presentation-v1`.
  2. `running_projects` contained 56 corrupted rows with `workspace_storage_path: NULL` and un-namespaced IDs.
  3. `resend_running_commands_for_instance` unilaterally executed `UPDATE running_projects SET is_running = 1`.
  4. In `PromptTreeViewModal.tsx`, opening the modal with `force = false` served the stale cached tree.
- **Solution:** Purged 56 corrupted rows from `running_projects`; purged stale cached trees from `prompt_tree_cache`; ensured `loadTree(true, true)` forces fresh tree generation on modal open; eliminated SQL fallbacks matching default on NULL/empty instance IDs; guarded empty workspace fallback.

### 1.5 Untitled Ghost Conversations with 0 Words
- **Root Cause:** 14 ghost conversations with 0 words and empty preview in `conversation_summaries.db` bypassed filtering when `is_running` was temporarily set or cached.
- **Solution:** Filtered out any conversation with `prompt_word_count === 0 && (!prompt_preview_200w || !prompt_preview_200w.trim())` unconditionally before rendering active lists or caching; merged remaining empty/stale conversations into a collapsible archived group.

### 1.6 Prompt View Header Sequence, Tail Snippet & Instance Trio
- **Root Cause:** Header lacked complete identity trio display and dual sequence formatting.
- **Solution:** Rendered prompt sequence badge `#{selectedConversation.seq_code || 'P001'}`, instance identity trio capsule `[#1 · Antigravity.exe · Default]`, and concluding prompt tail snippet `“… ending with: '<snippet>'”`.

---

## 2. Completed Subtasks & Verification Matrix

| Task-ID | Subtask Title | Target Files | Owner | Status | Evidence |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Task-01** | Architecture Specification, Component Spec & RCA | `02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/` | Lead & Spec Authors | `COMPLETED` | Authored `01-architecture-spec.md`, `02-component-spec.md`, `03-root-cause-analysis.md`, and 6 subtask files |
| **Task-02** | UI Line Gap Preservation via `<br>` & Markdown Formatting | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | `COMPLETED` | Replaced inline breaks with `<br className="my-1.5 block select-none" />` in renderer, parser, and raw view |
| **Task-03** | Interactive Dot-Dot (`...`) Click Expansion & Hotkey `N` / Send Now | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | `COMPLETED` | Interactive ellipsis toggle in preview and inspector modal; hotkey `N` and `Send Now` live dispatch with task file write and window focus |
| **Task-04** | Live Database Cache Purge & False Running Activity Elimination | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`, `src/pages/Instances.tsx` | Worker 02 | `COMPLETED` | Purged 56 corrupted rows and `prompt_tree_cache`; fixed SQL queries; guarded empty workspace fallback; updated process checks and `isDefaultOwned` |
| **Task-05** | Filter Out & Merge Untitled 0-Word Empty Conversations | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | `COMPLETED` | Filtered 0-word empty sessions unconditionally; auto-selection ignores ghost conversations |
| **Task-06** | Prompt View Header Sequence, Tail Snippet & Instance Trio | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | `COMPLETED` | Rendered prompt sequence `#{seq_code}`, tail snippet `… ending with: '{tailSnippet}'`, and instance identity trio `[#1 · Antigravity.exe · Default]` |
| **Task-07** | Live Testing & Running Prompts Properties Verification | `scratch/`, `repo_prompts.db` | Worker 02 | `COMPLETED` | Verified live DB: cleared cache, purged corrupted rows, confirmed white-presentation is not running under Default, validated `.antigravity_resume_task.json` writing, prompt moving, and running prompt properties |
| **Task-08** | Plan Consolidation & Single Atomic GitMap Push | `.ai-memory/plans/completed/` | Lead | `COMPLETED` | Consolidated plan, updated spec and plan indexes, checked secrets gate, completed task DB, and performed single atomic GitMap push |

---

## 3. Verification & Live Validation Summary

1. **Live Database Cleansing**:
   - `prompt_tree_cache` cleared.
   - 56 corrupted rows in `running_projects` purged.
   - `white-presentation-v1` verified 0 running records under Default.
2. **Task Resume JSON Generation**:
   - Verified `.antigravity_resume_task.json` written with model, prompt payload, and timestamp.
3. **Prompt Movement & Properties**:
   - Verified prompt movement across instances with cache invalidation and sequence numbering.
   - Verified 12-word prompt tail snippet extraction and word counts.
4. **Code Quality & Guidelines**:
   - Newline normalization clean across all modified files.
   - Boolean guidelines conformant.
   - Secrets gate verified 0 leaks.

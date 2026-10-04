# Master Plan: 128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix

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
- **Solution:** Inject block-level select-none `<br className="my-1.5 block select-none" />` elements on all newlines in `RichMarkdownRenderer`, `formatPromptForMarkdown`, and Raw View tab to enforce true vertical line spacing everywhere.

### 1.2 Interactive Dot-Dot (`...`) Click-to-Expand
- **Root Cause:** Truncated text ending with `...` inside lists, headings, and code blocks was flushed before paragraph parsing, and inspector modals lacked `onToggleExpand` forwarding.
- **Solution:** Support `isTruncated`, `showAllWords`, and `onToggleExpand` universally; bind interactive click handlers on the ellipsis token `[... [Expand Full Text]]` / `[... [Collapse]]` in preview, raw, and inspector modals.

### 1.3 `N` Key Hotkey & "Send Now" Resend Live Execution
- **Root Cause:** Missing active conversation/project resolution caused silent returns; `get_dispatched_prompts_cache` blocked previously sent signatures; missing focus calls left the IDE in background.
- **Solution:** Add automatic fallback resolution to newest non-empty conversation; write `.antigravity_resume_task.json`; trigger `resume_recent_project_prompts`; copy prompt to clipboard; focus target IDE window with toast feedback; verify live file generation.

### 1.4 Stale DB Cache & False Running Activity on Default Instance (`white-presentation-v1`)
- **Root Cause:** 
  1. `prompt_tree_cache` in `repo_prompts.db` retained stale cached JSON (`tree:all:50:false`) with `is_running: true` for `white-presentation-v1`.
  2. `running_projects` contained 56 corrupted rows with `workspace_storage_path: NULL` and un-namespaced IDs.
  3. `resend_running_commands_for_instance` unilaterally executed `UPDATE running_projects SET is_running = 1`.
  4. In `PromptTreeViewModal.tsx`, opening the modal with `force = false` served the stale cached tree.
- **Solution:** Purge corrupted rows from `running_projects`; purge stale cached trees from `prompt_tree_cache`; ensure `loadTree(true, true)` forces fresh tree generation on modal open; eliminate SQL fallbacks matching default on NULL/empty instance IDs.

### 1.5 Untitled Ghost Conversations with 0 Words
- **Root Cause:** 14 ghost conversations with 0 words and empty preview in `conversation_summaries.db` bypassed filtering when `is_running` was temporarily set or cached.
- **Solution:** Filter out any conversation with `prompt_word_count === 0 && (!prompt_preview_200w || !prompt_preview_200w.trim())` unconditionally before rendering active lists or caching; merge remaining empty/stale conversations into a collapsible archived group.

### 1.6 Prompt View Header Sequence, Tail Snippet & Instance Trio
- **Root Cause:** Header lacked complete identity trio display and dual sequence formatting.
- **Solution:** Render prompt sequence badge `#{selectedConversation.seq_code || 'P001'}`, instance identity trio capsule `[#1 · Antigravity.exe · Default]`, and concluding prompt tail snippet `“… ending with: '<snippet>'”`.

---

## 2. Granular Subtask Decomposition

| Task-ID | Subtask Title | Target Files | Owner | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Task-01** | Architecture Specification, Component Spec & RCA | `02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/` | Lead & Spec Authors | IN_PROGRESS |
| **Task-02** | UI Line Gap Preservation via `<br>` & Markdown Formatting | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | QUEUED |
| **Task-03** | Interactive Dot-Dot (`...`) Click Expansion & Hotkey `N` / Send Now | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | QUEUED |
| **Task-04** | Live Database Cache Purge & False Running Activity Elimination | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`, `src/pages/Instances.tsx` | Worker 02 | QUEUED |
| **Task-05** | Filter Out & Merge Untitled 0-Word Empty Conversations | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | QUEUED |
| **Task-06** | Prompt View Header Sequence, Tail Snippet & Instance Trio | `src/components/instances/PromptTreeViewModal.tsx` | Worker 01 | QUEUED |
| **Task-07** | Live Testing & Running Prompts Properties Verification | `scratch/`, `repo_prompts.db` | Worker 02 | QUEUED |
| **Task-08** | Plan Consolidation & Single Atomic GitMap Push | `.ai-memory/plans/completed/` | Lead | QUEUED |

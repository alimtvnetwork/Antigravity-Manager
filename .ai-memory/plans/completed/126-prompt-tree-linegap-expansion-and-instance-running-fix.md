# Completed Plan: 126-prompt-tree-linegap-expansion-and-instance-running-fix

## User Request (Verbatim)

```text
Okay. So here, if you look into this, the UI does not look okay. So first of all, the line gap, you don't have the line gap. How you get the text, you don't have the line gap. Fix the line gap in terms of display and everywhere. Okay? That's the first. Second, the issue here, because the line gap, you need to fix it with a BR tag only. That's the first thing. Okay. So apply that, and then format it for the markdown. Okay. Then when I click on dot dot, it does not expand to the full text. That is a problem, and also N key, or also Send Now does not work at all. So you need to test it live here, then it goes there, and you can trace back. And also the activity, like where the project is running or things like that. That is also incorrect. For example, if I give you the screenshot of the defaults, the default instance does not have anything but Integrab be running. But it says what presentation is running. This is absolutely wrong. So you have to find the root cause to see why it is happening. And if we go into the `gitmap` section. You want to get into the `gitmap` section, the Git maps prompt. If we open, the problem here is that it still has prompts which has untitled conversation and has no prompt content. I asked you not to show this in the list while you are seeing this. And also, in this UI, the prompt view, I need to see a little bit of prompts sequence, prompts ending text, and also the name of the instance of the `Id`. So again, `Id` sequence, `Id` exe name, and also `Id` instance name. These three things I need to see here in the UI nicely. You need to move one prompt. You need to check the running prompts and also the running prompts property, sending the running prompts, entering the prompt. You need to check all this. It's typically working in the past, but in your case it is not working. Lots of untitled conversations. I asked you to merge these untitled conversations which has no prompt. Try to merge this. Yet understood

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
10. Update the UI to display prompt sequence, prompt ending text, and the name of the instance of the `Id`.
11. Check and ensure the running prompts and their properties are functioning correctly.
12. Merge untitled conversations that have no prompt content.
```

---

## Completed Tasks & Architectural Outcomes

### 1. Specification & Planning (Task-01)
- Authored canonical architectural specification: `02-spec/21-app/126-prompt-tree-linegap-expansion-and-instance-running-fix/01-architecture-spec.md`.
- Authored component specification: `02-spec/21-app/126-prompt-tree-linegap-expansion-and-instance-running-fix/02-component-spec.md`.
- Authored 4-part root cause analysis (RCA): `02-spec/21-app/126-prompt-tree-linegap-expansion-and-instance-running-fix/03-root-cause-analysis.md`.
- Registered specifications in `02-spec/21-app/readme.md` and plans in `.ai-memory/plans/readme.md`.

### 2. UI Line Gap Preservation via `<br>` & Markdown Formatting (Task-02)
- Normalized all line endings in `formatPromptForMarkdown` (`\r\n` and `\r` to `\n`).
- In `RichMarkdownRenderer`, replaced collapsing inline breaks with `<br className="my-1.5 block select-none" />` for true vertical block gaps between lines and paragraphs.
- In the Raw View tab, replaced collapsed single lines with explicit block lines and `<br className="my-1.5 block select-none" />`.

### 3. Interactive Dot-Dot (`...`) Click Expansion & Hotkey `N` / "Send Now" (Task-03)
- Passed `isTruncated`, `showAllWords`, and `onToggleExpand` across `RichMarkdownRenderer` and the full inspector modal.
- Made trailing ellipsis interactive: clicking `...` or the inline button toggles between `[... [Expand Full Text]]` and `[... [Collapse]]`.
- Strengthened `handleResendPrompt` with fallback resolution: if `selectedConversation` is unselected, it automatically selects the latest non-empty conversation for the project; if `selectedProject` is unselected, it resolves from tree nodes.
- Wrote `.antigravity_resume_task.json` using `save_text_file`, triggered `invoke('resume_recent_project_prompts')`, copied prompt to clipboard, and called `focusOrLaunchInstance`.
- Hotkey `N` listener verified and wired to `handleResendPrompt`.

### 4. Backend Running Detection & False Positive SQL Fix (Task-04)
- Fixed `compute_project_conversation_tree` fallback trap in `src-tauri/src/modules/repo_db.rs`: when `has_conv_nodes` is false, it no longer blindly falls back to `is_prompt_running_for_project` without evidence; returns `(false, "IDLE_EMPTY_WORKSPACE")`.
- Eliminated SQL fallback trap: removed `(instance_id IS NULL OR instance_id = '')` matching default instance across queries in `repo_db.rs`. Strictly matches `(?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__'))`.
- Fixed Gate 0 in `is_prompt_running_for_project` to inspect default instance PIDs specifically.
- Ensured non-null `workspace_storage_path` and composite namespacing `{base_id}__{instance_id}` in `save_or_requeue_prompt` and `resend_running_commands_for_instance`.
- Added cleanup for legacy un-namespaced rows with `workspace_storage_path IS NULL`.
- In `src-tauri/src/modules/instance.rs`, excluded manager binaries (`agm.exe`, `Antigravity-Manager.exe`) from IDE process discovery.
- In `src/pages/Instances.tsx`, enforced strict instance ownership in `isNodeOwnedByInstance`.

### 5. Filter & Merge Untitled Zero-Content Conversations (Task-05)
- In `PromptTreeViewModal.tsx`, enforced strict ghost conversation detection: any conversation titled untitled/new conversation with 0 words and empty prompt preview is treated as ghost.
- Removed `if (Boolean(conv.is_running)) return false;` bypass in `isStaleOrEmptyConversation` for 0-word empty sessions.
- In `performAutoSelection`, prevented auto-selection of ghost or 0-word empty conversations.
- Cleanly aggregated stale/empty conversations into the collapsible `"Archived / Stale Prompts"` group.

### 6. Prompt View Header Sequence, Tail Snippet & Instance Trio (Task-06)
- Rendered prompt sequence badge `#{selectedConversation.seq_code || 'P001'}`.
- Rendered Instance Identity Trio capsule prominently: `[#{instanceSeqNum} · {instanceExeName} · {instanceNameDisplay}]` (e.g. `[#1 · Antigravity.exe · Default]`).
- Rendered trailing prompt snippet `“… ending with: '${tailSnippet}'”`.

### 7. Verification & Linter Gates (Task-07 & Task-08)
- Ran `python 03-ai-scripts/05-guideline-autofixer.py` across spec and subtask files (`exit 0`).
- Secrets gate clean: 0 hits across all files via `gitmap aum search -r`.
- Single atomic commit via GitMap.

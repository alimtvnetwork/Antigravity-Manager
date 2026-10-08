# Plan: 145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening

**Task Slug:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening`  
**Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)  
**Protocol:** `execute-parent-task-with-n-steps-v6` (A = 2, H = 2)  
**Status:** `IN PROGRESS`  

---

## User Request (Verbatim)

```text
# High Priority Instruction

You can see that this is a serious bug where I have my prompts running, but I don't have the results of anything that is running there. I could not see the prompts, and I could not open the prompts in the main window or whatever I have sent. And there are prompts which I have not executed. Can this distinguish what is prompt, what is not prompt? Because some of those are prompt, some of those are not. It's by the AI instruction. So we need to distinguish those, and those could be a sub point or things like that that you need to work on in the UI. I do see that the prompts bit showed up, but still the header display for the prompts tree view, it's very much bad. It does not have the UI/UX. Seems like broken stuff. The menus are not compact, buttons are not compact. I do see that you have most of the text displayed. That's all right. Appreciate it. You should have export button as well, copy button as well. Copy with image. If I do that, can go there. I don't have the image. Images are also there. That's nice. That's good. The copy button is working, but still, it looks broken, especially the header section. That needs to be compacted, the design issue and things like that, you need to work on it. And also there are things which are not prompt, but showing up. You need to work on it. Also, the preview you need to improve. In between, it shows truncated bytes, which we need to fix in terms of the display, I think. Can you help with that? Improve the preview. And also, if a prompt is running, we need to have some indicator that it is a running prompt or in queue prompt. We need to have that. It is not there yet. So I think you need to look back and look deep, the prompt tree view, how the prompts are collecting, taking the prompts backup, restoring those. I think these are still very limited. You need to work on it very hard. Try to find the root cause. Try to write the spec by yourself, in details. Like the testing, UI/UX specs, make it more professional.
```

---

## Subtask Breakdown

1. **Subtask 01:** [01-prompt-tree-header-and-toolbar-compaction-ui.md](./subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/01-prompt-tree-header-and-toolbar-compaction-ui.md)
   - Redesign modal header and preview toolbars into segmented dark-glass capsules per `AGENTS.md`.
   - Compact buttons, elimitate loose margins, integrate window controls and instance trio badge.
2. **Subtask 02:** [02-prompt-origin-distinction-and-subagent-hierarchy.md](./subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/02-prompt-origin-distinction-and-subagent-hierarchy.md)
   - Distinguish root User Prompts vs AI Subagent Instructions vs Non-Prompt system turns.
   - Support nested sub-point hierarchy and visual role badges (`User Prompt` vs `AI Subagent Instruction`).
3. **Subtask 03:** [03-running-and-queue-indicators-with-live-results.md](./subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/03-running-and-queue-indicators-with-live-results.md)
   - Add glowing animated emerald pulse badges for running prompts and amber badges for queued prompts.
   - Stream latest execution step results into active prompt preview pane.
   - Harden "Focus IDE" / "Open IDE Window" workspace activation.
4. **Subtask 04:** [04-truncation-banner-and-rich-copy-export.md](./subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/04-truncation-banner-and-rich-copy-export.md)
   - Parse and transform raw `<truncated N bytes>` markers into sleek dark-glass callout banners.
   - Implement dedicated Export (.md / .json) and "Copy with Image" rich clipboard support.
5. **Subtask 05:** [05-prompt-collection-backup-restore-hardening-and-e2e.md](./subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/05-prompt-collection-backup-restore-hardening-and-e2e.md)
   - Harden backend prompt collection, backup serialization, and FIFO restore queue.
   - Author comprehensive E2E test verifying full flow.

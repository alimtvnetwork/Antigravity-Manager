# Plan: UI/UX Improvements and Bug Fixes

## User Request (Verbatim)
```text
The UI coloring is now terrible. I'm not sure what you have done from the 4.248. 4.248 has a very good color in this section, and what you have done recently, button and everything just went away. The send and enqueue button is working, which is a very good thing. Rerun, I think it still has some issue inside the prompt tree, so try to focus on that. And also I cannot see anything. And also there is one more issue. If the prompt is running, let's say, and we already open the tree view, then it does not sync automatically. Probably I have to wait 30 seconds or reopen. That's one thing. That's correct. That's not a problem. If that is what it is happening, that's all right. Also, one more point that I wanted to point out is that Telegram communication is slow. Can we speed up this? Okay, it's extremely slow. You can test it. Let's say prompt inspection or prompt sending. You can try it by step by step testing what is going on. So that needs to be fixed in order to make things better. And also the hover over of the scoreboard, that also looks very terrible. You need to fix it. So make sure you understand the UI/UX issue. The previous version, which is like the 4.248 was good. 247 was also good. Okay, you can compare what you have done. Make sure you focus on the cleaning, so that the color and buttons, it looks more professional. For example, here on the top level of the header, the icons doesn't look like it's fit. It is done by a UI/UX expert. So, so many issues I think you need to fix. Okay. Hover colors needs to have a little bit of dark or... You can show me HTML first, which I can verify by giving you more feedbacks. That would be easier. And then you can start working on it. Okay. Is it clear?
```

## Architecture Roadmap & Subtask Allocation
1. **Subtask-01: UI Coloring & Theme IACVT Invalidation Remediation**
   - File: `.ai-memory/plans/subtasks/155-ui-ux-improvements-and-bug-fixes/01-ui-coloring-and-theme-remediation.md`
   - Scope: Fix invalid arbitrary Tailwind opacity classes (`dark:bg-[var(--app-bg)]/95`, `dark:bg-[var(--app-bg)]/60`, `dark:bg-[var(--app-surface)]/95`) in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `TreePanel.tsx`, and `DetailHeader.tsx`. Restore dark-glass styling to modal container, sidebar, and detail views.
2. **Subtask-02: Scoreboard Hover Candidate Card Contrast & Popover Polish**
   - File: `.ai-memory/plans/subtasks/155-ui-ux-improvements-and-bug-fixes/02-scoreboard-hover-card-polish.md`
   - Scope: Fix `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` popover container to use authentic dark-glass backdrop blur and slate borders, eliminating white-on-white text collapse.
3. **Subtask-03: Header Icons Capsule & Segmented Nav Polish**
   - File: `.ai-memory/plans/subtasks/155-ui-ux-improvements-and-bug-fixes/03-header-icons-capsule-refinement.md`
   - Scope: Refine `src/components/navbar/NavMenu.tsx` icon capsule into a clean segmented pill with unified tinting, subtle dividers, and dark hover states.
4. **Subtask-04: Prompt Tree Auto-Sync & Rerun Logic Fixes**
   - File: `.ai-memory/plans/subtasks/155-ui-ux-improvements-and-bug-fixes/04-prompt-tree-autosync-and-rerun-fix.md`
   - Scope: Implement dynamic 2–3s high-frequency polling during active tasks in `usePromptTreeData.ts` with `force: true` cache bypass. Fix rerun nudge message truncation and self-cancelling preemption loop in `usePromptTreePromptActions.ts`. Preserve Send and Enqueue buttons.
5. **Subtask-05: Telegram Communication Speed Optimization**
   - File: `.ai-memory/plans/subtasks/155-ui-ux-improvements-and-bug-fixes/05-telegram-speed-optimization.md`
   - Scope: Remove 60s startup sleep in `daemon.rs`, make prompt dispatch reply immediate with background proof shot capture, add in-memory cache for cluster node lookups, and fire fast typing actions.

## Interactive Verification HTML Prototype
- Artifact: Live interactive HTML preview rendered for user verification and feedback before code modifications.

---
Closure (2026-10-23, subtask 190-05): DONE-IN-CODE per research 01 audit of task 190; UI/UX fixes verified present in the codebase. Filed to completed.

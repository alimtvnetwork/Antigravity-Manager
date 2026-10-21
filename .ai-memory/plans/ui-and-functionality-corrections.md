# Master Plan: UI and Functionality Corrections

- **Task Name**: UI and Functionality Corrections
- **Slug**: `ui-and-functionality-corrections`
- **Spec Directory**: `02-spec/21-app/ui-and-functionality-corrections/`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/ui-and-functionality-corrections/`
- **Author**: Maintainer Team (@aukgit)
- **Status**: IN PROGRESS
- **Budget**: 400 steps (Phase 1: 1..100, Phase 2: 101..250, Phase 3: 251..400)
- **Concurrency**: A = 3, H = 2, Capacity = 6

---

## User Request (Verbatim)

```text
Send button still doesn't work, why? have you tested with e2e

[Attached Images 1-13]

# UI and Functionality Corrections: high priority instruction, non-negotiable task

The sending prompts to the screen from the prompt tree. That button works, but that button does not work as expected. The Enqueue button does not work. The Send button is doing Enqueue button's work. Send means whatever task is running at the moment, it would be cancel and the current prompt which I'm sending, that should be executed right now. That should be the Send button. The Enqueue should be currently what the Send button is doing. That should be the Enqueue button. Please correct it. I have pointed this out several times. In the prompt tree view, the left-hand side, the tree view, I should be able to reduce the width at any time. Try to have that expandable behavior. The UI looks much more compact now compared to the past. It is a good work. The images are also going as a file name, which is good, not bad. The rename, let me test out if the rename works. Rename does not work. If I double-click and rename, it does not affect to the IDE. Check that you are not syncing with the IDE. That is the main problem. Renaming of the title. I'm going to give you again the screenshot. Try to find the root cause, try to fix it. Other than that, so far the work is good. Appreciable. Make sure at the end you do a minor bump and release after doing the end-to-end testing on these three features, as I have mentioned. Also, can you improve the UI a little bit better, based on the themes? If I go into the One Dark Pro, the theme still actually has purple things. If I go into other colors, the middle portion is always purple. It does not change based on that. Hover and other effects should be changing based on the theme that we pick. It is not happening. The cards, it is actually changing in movement, like position. Just makes things bigger or a little bit slides of the positioning. Never change the positioning of the item on hover, change the coloring. With a CSS3 animation, like 0.3 or 0.5 seconds so that it feels very nice. You can play a contrast role, like the background can be a little bit lighter, the text color can be darker. This could be one way. I'm not saying that you have to apply and test all the time, but this is just an idea. For the scoreboard, if I hover over to the above level, some of the header actually gets stuck with the above level header, as you can see in the screenshot I have given you. Make sure you fix all these things. Make sure you do a release with a minor bump. Also, how much coding guideline is followed? Can you please give me a percentage right now? How much we should improve as well, based on the current code base, how much the improvements areas are. Please focus on this.
```

---

## Architecture & Work Breakdown

### Subtask Index
1. `01-send-cancel-preemption-and-enqueue-staging.md`: Invert Send vs Enqueue logic. "Send" immediately cancels any currently active/running task on the IDE instance and forces execution of the current prompt. "Enqueue" stages the prompt into FIFO queue without cancelling.
2. `02-ide-title-rename-synchronization.md`: Implement bidirectional conversation title renaming sync with Antigravity IDE storage (`state.vscdb`, workspaceStorage, conversation transcripts, metadata).
3. `03-tree-view-resizable-splitter.md`: Add draggable, expandable/collapsible splitter handle between TreePanel and Detail panel with localStorage width persistence.
4. `04-theme-adaptability-and-hover-transitions.md`: Purge hardcoded purple values, support dynamic One Dark Pro and custom theme tokens, replace displacing hover animations with smooth CSS3 color/contrast transitions (`transition: all 0.3s ease`).
5. `05-scoreboard-header-tooltip-fix.md`: Fix scoreboard formula hover popover/tooltip clipping, z-index elevation, and collision containment against upper table headers.
6. `06-e2e-testing-and-minor-bump-release.md`: Run targeted E2E tests for Send preemption, IDE renaming, and resizable tree; perform minor version bump; update documentation and changelogs.

---

## Invariants & Guardrails
- **Search Ban**: GitMap exclusively (`gitmap find`, `gitmap aum search`, `gitmap cat`). Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Paths**: Strictly relative repository Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **Attribution**: Strictly `@aukgit` across all release notes and changelogs.
- **Zero Secrets**: Never commit tokens, credentials, or private machine IP addresses.

# Master Execution Plan: 225-prompt-tree-view-issues

- **Specification Directory:** [02-spec/21-app/225-prompt-tree-view-issues/](../../02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md)
- **Subtasks Directory:** [.ai-memory/plans/subtasks/225-prompt-tree-view-issues/](../subtasks/225-prompt-tree-view-issues/)
- **Task ID:** `225`
- **Slug:** `prompt-tree-view-issues`
- **Status:** `COMPLETED`
- **Date:** 2026-10-24

---

## User Request (Verbatim)

```text
cargo and rust is installed try to build and do e2e tesing you stupid fuck

WHY you are not doing the E2e test by building the code and release properly??

# Prompt Tree View Issues: high priority instruction, non-negotiable task

Hi there. You have a couple of serious issues on the prompt tree view. First of all, the left-hand tree view takes a lot of space. Try to have header portion to be minimized, that's the first thing, and try to reduce the tree spacing. And also the images. I don't see the images are correctly there. It feels a bit buggy, and the way that you pass the image, that's not really very helpful. The reason why I'm saying is because when you pass the image, you can use the file format for the IDE because you are giving absolute path, and then we can click on the image and view the image. Currently, the path, we cannot view it by clicking on it. That's one issue. So make sure that you pose it correctly so that IDE also display as a file, and we click and view it. That's one thing. The next thing is that please check the correctness. I do believe that your image position or capturing position is not correct. And also same as your conversation name and conversation prompts. These are not correct. You need to do the end-to-end testing here. And also, when I hover over to any conversation, there is a green line appears on top and sides. That actually breaks the UI positioning. That's really terrible. You need to fix it. You don't need to have the green bar on top. If you're refreshing or something like this, you need to do it under the hood. And why you are refreshing when hover over, I don't understand, because in most cases, when you generate this tree, you should have caching data. So why you are refreshing? So you should only refresh in 30 seconds or based on the Refresh button on top. And the Refresh button has very close button besides it, yellow button, which needs to written as Reset, I think. So import/export, make it a icon. Remove the text created on hover over. The hide tree view also make icon, so you will have space, the Reset and the Refresh type of text there, and also hover tool tip. And the tree view will only be updated in next 30 seconds as per the configuration, not before that. And in the Send place, Send button has a dropdown in the past. Send and Enqueue should be together. Send, Enqueue, Rerun. These three should be together as a dropdown, which you actually take it away. And in this place, we could actually set the new conversation or current conversation, which is very wrong. When I send it, even though it's in current conversation, it sends to the new conversation. That is a bug. You need to do the root cause analysis and try to fix it. It's been several times that I'm trying to understand what went wrong, but it turns out that it is not respected properly. Can you please use the cargo and build and do the end-to-end testing with all this stuff? From the CLI, we should have all these options to test as well. If not, then enhance the CLI methods. At the end, your job is to minor bump and release at the end. So check it very carefully and then do it, because it's been several times that I'm asking for it. And also in the button section on the root level. So the buttons are separated by the dashes, so it should be full space. It's not circular in between, which is a serious bug which you need to fix, actually. So you did not respected that. Also, if I get down, in many places, the error manage is not respected. And there are several errors in the dashboard that I could see. I hope you can use the CLI to get these method errors and fix those. And if you cannot get this, then you need to add CLI methods in order to do that, and also add those as a skill so that any AI in the future can adapt to it. I hope it makes sense. Try to make sure that the fast forwarding or smart switch works properly and the hover buttons make it better. Both of the hover buttons, the background of the yellow color, I really like it, but it didn't do the contrast. That means the color, white color, it means white or black so that it played a contrast role. And also with, let's say, refresh buttons, it's very much the same as the hover over, so it just blends in. It does not look very nice. So make sure the buttons has a very nice 3D behavior everywhere, and revalidate the switching logic which I have given last night. Check in spec. You can verify the end-to-end logic so that it works. Please confirm that you understood everything and you can work on it. Does this make sense? Wait a minute. So I hope you understood. You can do all this end-to-end testing now, and make a minor bump and release so that everything is working. Is it clear?

## task: Prompt Tree View Issues
## slug: prompt-tree-view-issues

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/XX-prompt-tree-view-issues/ and enqueue plan task in .ai-memory/plans/pending/XX-prompt-tree-view-issues.md (subtasks in .ai-memory/plans/subtasks/XX-prompt-tree-view-issues/) first, (xx should be figured out by the folder structure and running AI tools, xx is sequence 01, 02)
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Minimize the header portion and reduce tree spacing in the left-hand tree view.
6. Correctly display images using file format for IDE with clickable paths.
7. Conduct end-to-end testing for image position, conversation names, and prompts.
8. Remove green line on hover over conversations to fix UI positioning.
9. Implement caching to prevent unnecessary refreshes; refresh only every 30 seconds or via Refresh button.
10. Change the label of the yellow button next to Refresh to "Reset".
11. Convert import/export and hide tree view text to icons.
12. Ensure Send, Enqueue, and Rerun are together in a dropdown.
13. Fix bug where messages are sent to new conversation instead of current.
14. Use cargo to build and perform end-to-end testing.
15. Enhance CLI methods if necessary for testing.
16. Perform a minor version bump and release after testing.
17. Fix button spacing and ensure full space separation.
18. Address error management issues in the dashboard.
19. Add CLI methods and skills for error management.
20. Improve hover button contrast and 3D behavior.
21. Revalidate switching logic as per the spec.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing work for improved work or less friction, faster execution for AI and coding without errors and following Coding Guidelines.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Track in-flight file touches and prevent write collisions using GitMap RC Heatmap (`gitmap rc heatmap status` / `gitmap rc heatmap tree` / `gitmap rc heatmap live` / `gitmap rc heatmap sync`).
- Only add relative paths, never add absolute paths during work; this must be respected on release page and in release notes as well.
```

---

## 1. Specification & Subtask Index

- **Master Audit Ledger:** [02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md](../../02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md)
- **Subtasks Directory:** [.ai-memory/plans/subtasks/225-prompt-tree-view-issues/](../subtasks/225-prompt-tree-view-issues/)
  1. `01-header-and-tree-compactness.md` (Header minimization, tree spacing, icon buttons, Reset label, 30s cache guard)
  2. `02-hover-artifacts-and-image-links.md` (Eliminate green hover line, stop hover refetching, clickable IDE file:// image links, position accuracy)
  3. `03-send-enqueue-rerun-dropdown-and-conversation-target.md` (Restore [Send, Enqueue, Rerun] dropdown, RCA & fix for Current vs. New Conversation misrouting)
  4. `04-button-capsules-contrast-and-cli-error-management.md` (Contiguous segmented pill capsules, yellow button contrast, 3D tactile elevation, CLI error inspector command & skill)
  5. `05-e2e-testing-cargo-build-and-minor-bump-release.md` (Cargo release build, comprehensive E2E tests, switching logic revalidation, atomic minor bump & release)

---

## 2. Multi-Agent Delegation Plan (A = 3, H = 2, Capacity = 6)

```text
Tree Hierarchy:
Root Orchestrator (Lead Agent)
├── Worker 01 (Frontend Tree & Header Specialist)
│   ├── Compact header, icon buttons, Reset label, tree spacing
│   └── Eliminate green hover line, enforce 30s polling guard
├── Worker 02 (Prompt Actions & Conversation Target Specialist)
│   ├── Combined [Send, Enqueue, Rerun] dropdown
│   └── RCA & fix for Current vs. New Conversation bug
└── Worker 03 (Image Formatting, Error Management & Release Specialist)
    ├── Clickable IDE file:// links & media position correctness
    ├── Pill capsule button styling & high-contrast yellow states
    └── CLI error management methods, E2E testing, minor bump
```

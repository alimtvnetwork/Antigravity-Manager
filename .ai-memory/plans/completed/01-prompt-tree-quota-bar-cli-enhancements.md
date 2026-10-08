# Completed Plan: 01-prompt-tree-quota-bar-cli-enhancements

## Canonical Specification Reference
- Specification: [01-architecture-spec.md](../../02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/01-architecture-spec.md)
- Component & Visual Spec: [02-component-spec.md](../../02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/02-component-spec.md)
- AGM CLI 35-Command Roadmap: [03-agm-cli-roadmap-spec.md](../../02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/03-agm-cli-roadmap-spec.md)
- Secure REST API Gateway Spec: [04-rest-gateway-spec.md](../../02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/04-rest-gateway-spec.md)
- E2E Verification Report: [05-e2e-testing-report.md](../../02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/05-e2e-testing-report.md)

## User Request (Verbatim)

```text
# High Priority Instruction

https://prnt.sc/_dAZ8hP0KPgZ
https://prnt.sc/LYS8VqaHwMOF


You can see that this is a serious bug where I have my prompts running, but I don't have the results of anything that is running there. I could not see the prompts, and I could not open the prompts in the main window or whatever I have sent. And there are prompts which I have not executed. Can this distinguish what is prompt, what is not prompt? Because some of those are prompt, some of those are not. It's by the AI instruction. So we need to distinguish those, and those could be a sub point or things like that that you need to work on in the UI. I do see that the prompts bit showed up, but still the header display for the prompts tree view, it's very much bad. It does not have the UI/UX. Seems like broken stuff. The menus are not compact, buttons are not compact. I do see that you have most of the text displayed. That's all right. Appreciate it. You should have export button as well, copy button as well. Copy with image. If I do that, can go there. I don't have the image. Images are also there. That's nice. That's good. The copy button is working, but still, it looks broken, especially the header section. That needs to be compacted, the design issue and things like that, you need to work on it. And also there are things which are not prompt, but showing up. You need to work on it. Also, the preview you need to improve. In between, it shows truncated bytes, which we need to fix in terms of the display, I think. Can you help with that? Improve the preview. And also, if a prompt is running, we need to have some indicator that it is a running prompt or in queue prompt. We need to have that. It is not there yet. So I think you need to look back and look deep, the prompt tree view, how the prompts are collecting, taking the prompts backup, restoring those. I think these are still very limited. You need to work on it very hard. Try to find the root cause. Try to write the spec by yourself, in details. Like the testing, UI/UX specs, make it more professional. Then you work on it, make sure that everything is achieved. And also, if a prompt is repeated. For example, in this case, I do see a fix pipeline that is repeated several times, same prompt repeated. In these cases, I think you can make those as a grouping and display it. You need to work on it properly. There is that, and also the UI is still bad. That means the card section of the instances still looks very poor, and you did not work on it yet. So please get cool, and then work on it. And then finally, do end-to-end testing for sure. And then, how do you do the end-to-end testing? You create a new instance, do your testing, do send prompt, see the running prompt, switch the account, see the prompts are running. These are the things that I want you to verify each time. And also check the tree view is compacted, grouped together, and things like that. Is it clear?

Do you think we can increase or enhance the `Antigravity Manager (`Antigravity Manager (AGM)`)` CLI methods, commands, and verb following recent changes in the `gitmap`? Can you please make a great plan, like 30 to 40 improvements plans in the CLI level so that user feels comfortable working with CLI? And also make sure that we have the way to open the rest endpoints for most of the tasks that we wanted to do, and we should be able to have a secure pathway in the future. Is it clear? Do you understand the task? Can you please follow through?

Also in the latest version, let's say, the progress bar is a bit problematic. What do I mean by that? It has so many dots and checks, and colors are not even blended in. That is one of the problems. So what I want you to do is make sure that you have not more than five round balls. And the colors needs to be blended in, or with the back-end color, when it is on that position. Remember that. And the left-hand side would be a little bit more dark red, with white. And it should have a percentage number when it goes below, let's say, 25%. So those checks which should have the percentage number rather than check. So these are the things I want you to improve in the progress bar section. It does not look much professional, and you did not reduce the space with space in the scroll bar. It is taking too much spaces, and there are lots of white space, as you can see in the screenshot. So I think first you should write for the AI how it could improve the UI. So look into the screenshot, write for the AI so that it can follow the instruction, and then you fix the progress bar screenshot and everything else. Do you understand? And also make sure the theme and hover over effects are more professional. It is still not that professional yet.
```

## Consolidated Outcomes & Evidence

### 1. Prompt Tree View & Conversation Inspector (`src/components/instances/PromptTreeViewModal.tsx`)
- **Prompt Classification**: Implemented `getPromptCategory` tagging entries with distinct visual badges: `[SUBAGENT]` (purple badge for automated briefs/instructions), `[DRAFT]` (amber badge for pending tasks), and `[USER]` (cyan badge for human user prompts).
- **Segmented Dark-Glass Pill Capsule**: Converted scattered toolbar buttons into contiguous segmented capsules (`rounded-full`, shared border, subtle divider lines, dark-glass backdrop blur) per `AGENTS.md`.
- **Export & Copy With Images**: Implemented `handleCopyWithImages` generating dual `text/html` and `text/plain` clipboard blobs, and `handleExportPrompt` for exporting Markdown/JSON files with full frontmatter.
- **Repeated Prompt Grouping**: Added prompt deduplication clustering consecutive identical prompts (e.g. repeated `"Fix Pipeline Error And Bump"`) with repeat count pills `(xN)`.
- **Pulsing Activity Indicators**: Integrated animated glowing green (`RUNNING`) and amber (`QUEUED`) badges in tree view nodes and inspector headers.
- **Preview Sanitization**: Eliminated raw `<truncated \d+ bytes>` and byte corruption artifacts.

### 2. Quota Progress Bar Modernization (`src/components/accounts/QuotaProgressBar.tsx`, `src/components/common/WaterDrainProgressBar.tsx`)
- **Milestone Round Balls Cap**: Strictly capped milestone nodes to maximum 5 (`[100, 75, 50, 25]`), completely replacing legacy 11-dot layouts.
- **Color Blending**: Milestone nodes dynamically blend into the exact background track color at their horizontal position.
- **Critical Quota (< 25%) Dark Red & White**: Added transition to deep dark red gradient (`from-[#7f1d1d] via-[#991b1b] to-[#dc2626]`) with high-contrast white text when quota < 25%.
- **Percentage Numbers on Critical Checks**: When below 25%, checkpoint nodes dynamically display the exact percentage number (e.g. `18%`, `12%`) instead of checkmarks.
- **Neon Green Radiant Glow**: Retained user-accepted `#1af18d` neon emerald glow for healthy quotas (>= 75%).

### 3. Instance Cards Polish & Scrollbars (`src/pages/Instances.tsx`, `src/App.css`)
- **Whitespace Reduction**: Tightened card outer padding to `p-2.5` (`p-2` compact), eliminated dead space between account row, quota bars, and recent projects.
- **Custom Thin Scrollbar**: Implemented `.custom-thin-scrollbar` (4px width, transparent track, subtle hover thumb) preventing scrollbar clutter.
- **Theme & Hover Effects**: Added professional dark-glass elevation (`dark:hover:border-blue-500/40 dark:hover:bg-[#061421]`).

### 4. AGM CLI Commands & 35-40 Command Roadmap (`src-tauri/src/modules/cli.rs`)
- Added CLI subcommands:
  - `agm prompts ls` / `agm prompts list` (with `--instance`, `--all`, `--force`, `--json`)
  - `agm prompts tree` (hierarchical ASCII tree with `--json`)
  - `agm prompts backup` (split SQLite backup to `repo_prompts.db`)
  - `agm prompts restore` (with `--keep` flag)
  - `agm doctor` / `agm check` (5-subsystem health diagnostic with `--json`)
- Authored exhaustive 35-command catalog across 7 domains matching GitMap conventions.

### 5. Secure REST Gateway Architecture (`04-rest-gateway-spec.md`)
- Detailed Axum HTTP server architecture bound to `127.0.0.1:8045` with automatic security elevation upon LAN enablement, Bearer token authentication, and IP whitelisting.

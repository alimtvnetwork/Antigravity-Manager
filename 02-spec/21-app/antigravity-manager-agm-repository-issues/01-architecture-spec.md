# Architecture Specification: Antigravity Manager Repository Issues & Enhancements

Slug: `antigravity-manager-agm-repository-issues`
Status: IN PROGRESS
Date: 2026-10-17

## User Request (Verbatim)

```text
# Antigravity Manager (`Antigravity Manager (AGM)`) Repository Issues: high priority instruction, non-negotiable task

There are several issues in the `Antigravity Manager (`Antigravity Manager (AGM)`)` repository. First of all, in the Telegram section. When working with Telegram, it's nice these steps have been pulled off. A couple of serious issues. My machine has the older version of the Antigravity tools, which the Telegram is sending information to, v1.9.0. That is some root cause to investigate in the machine. In the `Antigravity Manager (`Antigravity Manager (AGM)`)`, there should be an other version clean command, or clean space other version, or OV. That would need CLI help to remove all other versions from the machine. Keep the account information inside the SQLite DB. Telegram can send the prompt, but it sends an ``Id`` with it, which is not necessary. It should just send the prompt given, and the screenshot is not correct. In Telegram, there is a summary of the projects and the title of the conversation, but a little bit of subpoint is needed. Prompts, let's say 200 words or 100 words, should be configurable from the settings UI and CLI. After selecting the project, previous prompts, a short version, let's say 100 words, 200 words, should be visible and resendable. In the instances, the list view goes right-hand side, which is not correct, and the data executable should be one column. The prompt button should be outside in the list view of the instance. The send button has an overlapping issue in the UI. The prompt tree does not have the updated prompt. The refresh button does not take the recent prompts to the UI. Proper synchronization is needed. The copy all with image feature is not working as expected. Closing the anti-gravity tool should end all processes. The progress bar does not have different color segments. In the log section, buttons like log should have a section called data-UI-name. Inside the log, there should be a data-UI ``Id``. The audit log should show a friendly name for buttons. The audit log summary should have a copy button. From the CLI, a signal can be sent to any AI, like `Antigravity Manager (`Antigravity Manager (AGM)`)` read logs. The CLI segment should be powerful. The send button from the prompts should be fixed. The synchronization of the prompts is not working as expected. The account switch functionality should be 15%, not 98%. Backing up the prompt, resending the prompt does not work. Enhance the UI for the prompt tree. Copy content with images should work, and buttons should be compacted. The UI should be more enticing. The updater UI needs improvement following `gitmap`. Add features for Telegram, fix the screenshot issue, add the clear issue of the other version. The latest version should be the default one. When updating to the older version, it should clean up the machine and revert to that older version. Keep the settings back in. Verify the code thoroughly. In the release note, remove the segment thanking aukgit. Check the PR from the other repo inside the code base. Provide critical feedback on changes. Confirm checking the fleet option for multiple machines. In the `Antigravity Manager (`Antigravity Manager (AGM)`)` log, filter out the system log. Find the root cause of the Telegram log issue and fix it if possible.
```

## Media & Visual Artifacts

- **Media 1**: `assets/screenshots/antigravity-manager-agm-repository-issues-01.png` - Overlapping UI between Prompt Instruction tab and Send/Enqueue dropdown controls in PromptTreeView.
- **Media 2**: `assets/screenshots/antigravity-manager-agm-repository-issues-02.png` - Progress bar requiring distinct color-segmented track styling.
- **Media 3**: `assets/screenshots/antigravity-manager-agm-repository-issues-03.png` - Disconnect between Copy All (Content + Images) and top-bar Copy All.
- **Media 4**: `assets/screenshots/antigravity-manager-agm-repository-issues-04.png` - Shared Cluster fleet status and SSH node health check verification.

## 1. Problem Classification & Root Causes

### 1.1 Telegram Version Divergence & Stale Binary Detection
- **Issue**: Telegram inbound worker reports version `v1.9.0` or contacts older versions installed in other paths.
- **Root Cause**: Windows systems may have legacy binaries in `%LOCALAPPDATA%\\Programs\\antigravity-tools`, `C:\\Program Files\\`, or `%USERPROFILE%\\.antigravity_tools` leftover from earlier installers.
- **Solution**:
  1. Add `agm versions clean` / `agm clean-other-versions` / `agm ov` CLI commands that detect and remove stale binaries from secondary directories while strictly preserving SQLite databases (`state.vscdb`, `accounts.db`, `task_index.db`).
  2. Verify Telegram inbound version reporting dynamically fetches `env!("CARGO_PKG_VERSION")`.

### 1.2 Telegram Prompt Formatting & Configurable Summaries
- **Issue**: Telegram appends raw ID tags to prompts; lacks subpoints with configurable 100/200-word summaries; cannot resend recent short prompts.
- **Solution**:
  1. Strip unnecessary ID wrappers when dispatching prompts via Telegram.
  2. Add `telegram_prompt_summary_words` (100 or 200, configurable via UI settings and `agm telegram config --summary-words <N>`).
  3. Render previous prompts as short numbered snippets with one-tap inline resend buttons.

### 1.3 Instance List View & Column Combination
- **Issue**: Instances list view overflows horizontally to the right; Exe path and Data path occupy separate wide columns; Prompt button is nested in the `...` menu.
- **Solution**:
  1. Combine IDE Exe & Data Directory into a single unified column showing the ending path segments (`Antigravity.exe` / `...\\default-copy-5597`).
  2. Expose the `Prompt` button directly in the action row alongside `Switch` and `Stop`.
  3. Enforce table bounding (`max-w-full overflow-x-auto`) to stop horizontal runaway.

### 1.4 Prompt Tree View UI Overlap & Synchronization
- **Issue**: `Prompt Instruction` / `AI Results & Outputs` capsule physically overlaps the Send dropdown in `DetailHeader.tsx` (Media 1); Refresh button fails to pull fresh turns from disk.
- **Solution**:
  1. Redesign `DetailHeader.tsx` layout into a two-row flex container: Row 1 holds metadata & action buttons; Row 2 holds View Mode pill capsules, eliminating collision.
  2. In `tree.rs` and `usePromptTreeData.ts`, ensure `Refresh` forces cache invalidation and directly rescans the workspace transcript.

### 1.5 Copy All With Images
- **Issue**: "Copy All (Content + Images)" fails to copy images properly onto clipboard (Media 3).
- **Solution**:
  1. Ensure HTML payload with valid `<img>` data URIs or base64 sources is written to `ClipboardItem({"text/html": ..., "text/plain": ...})`.
  2. Unify top and bottom Copy buttons so behavior is consistent.

### 1.6 Progress Bar Multi-Color Segments
- **Issue**: Progress bar lacks segmented color styling (Media 2).
- **Solution**:
  1. Render QuotaProgressBar track with distinct color segments across quintile intervals (0-20% Red, 20-40% Orange, 40-60% Amber, 60-80% Lime, 80-100% Emerald).

### 1.7 UI Attributes & Audit Copy
- **Issue**: Buttons lack `data-ui-name` and logs lack `data-ui-id`. Audit log summary lacks a copy button.
- **Solution**:
  1. Attach `data-ui-name` and `data-ui-id` to key buttons and logging elements.
  2. Add copy-to-clipboard button to the Audit log summary modal.
  3. Filter noisy system logs from the live AGM log viewer.

### 1.8 Process Exit Safety
- **Issue**: Closing Antigravity Manager should exit all background processes cleanly.
- **Solution**:
  1. Ensure `exit_app_fully` halts child threads, tunnels, daemons, and detached workers before calling `process::exit(0)`.

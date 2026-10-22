# Specification: UI/UX Improvements and Bug Fixes

## 1. Overview & Objective
This specification addresses UI/UX color regressions, interactive element styling, prompt tree auto-sync and rerun mechanics, scoreboard hover card contrast, header navigation icon aesthetics, and Telegram bot communication latency in Antigravity-Manager.

Recent styling changes in `v4.250.0` attempted to introduce theme adaptability using CSS custom properties with Tailwind opacity modifiers (e.g. `dark:bg-[var(--app-bg,#090611)]/95`). Because `--app-bg` and `--app-surface` store hex color codes, Chromium's CSS engine rejects `rgb(#hex / opacity)` as Invalid At Computed-Value Time (IACVT), cascading back to stark `bg-white`. This caused the Prompt Tree left sidebar and the Scoreboard candidate scorecard popover to render blinding white backgrounds with unreadable white-on-white text.

Furthermore, Prompt Tree auto-sync during running prompts suffered from a 30-second default polling interval and a 120-second SQLite cache TTL, the prompt tree rerun pipeline suffered from prompt text omission and self-cancelling preemption, and Telegram command dispatch was blocked by synchronous screenshot capturing, thread sleeps, and sequential process spawns.

---

## 2. Root Cause Analysis (RCA)

### 2.1 UI Color Invalidation & White Backgrounds
- **Defect**: The Prompt Tree left panel and Scoreboard candidate hover card displayed stark white backgrounds (`bg-white`) in dark mode.
- **Root Cause**: Commit `33ea5d83` introduced `dark:bg-[var(--app-bg,#090611)]/95` and `dark:bg-[var(--app-surface,#18181b)]/95`. Under W3C CSS Color Module 4, Tailwind compiles `/alpha` modifiers into `rgb(var(...) / alpha)`. Because `ThemeManager.tsx` assigns 6-digit hex values (`#071A27`, `#090611`, `#171025`) to these variables, `rgb(#hex / alpha)` is invalid CSS syntax at computed-value time. The browser discards the dark mode rule and falls back to `bg-white` and `bg-slate-50/50`.
- **Solution**: Replace invalid arbitrary variable opacity syntax with solid theme variables (`dark:bg-[var(--ui-canvas)]`, `dark:bg-[var(--ui-surface-1)]`, `dark:bg-[#0c0817]`, `dark:bg-[#13111c]`) and RGB triplets where variable opacity is required.

### 2.2 Scoreboard Hover Card Contrast & Popover Styling
- **Defect**: Hovering over candidate readiness scores in the Account table displays a stark white popover card with pale gray and pastel typography, rendering the formula and metrics illegible.
- **Root Cause**: `AlgorithmScoreCell.tsx` wrapped the popover in `createPortal` with `dark:bg-[var(--app-surface,#18181b)]/95`. Due to IACVT syntax failure, the popover defaulted to `bg-white/95`, while inner text retained dark-mode styling (`dark:text-slate-100`, `dark:text-slate-300`).
- **Solution**: Style the hover card with authentic dark-glass styling (`dark:bg-slate-900/95 backdrop-blur-md border border-slate-700/60 shadow-2xl text-slate-100`). Formula container styled with `dark:bg-slate-950/80 border border-slate-800`.

### 2.3 Prompt Tree Auto-Sync During Active Tasks
- **Defect**: When a prompt is actively running in an IDE, the opened Prompt Tree does not update automatically until 30 seconds pass or the modal is closed and reopened.
- **Root Cause**: `usePromptTreeData.ts` enforces a 30s default timer (`Math.max(intervalMs, 15000)`), and backend `get_project_conversation_tree_cached` in `src-tauri/src/modules/repo_db/tree.rs` enforces a 120-second TTL when `force: false`.
- **Solution**:
  1. Dynamically downshift the polling interval to 2–3 seconds whenever an active running prompt is detected (`is_running: true` or active `runningItem`).
  2. Pass `force: true` during active polling or active state transitions so the SQLite cache is refreshed immediately.
  3. Bridge `useRunningAndUpNext` state transitions directly into `loadTree(false, true)`.

### 2.4 Prompt Tree Rerun Mechanics
- **Defect**: Triggering Rerun inside the Prompt Tree creates empty prompts or fails silently.
- **Root Cause**:
  1. `buildRerunPrompt` ignored `customNudge`, and passing string input left `customNudgeMessage` empty. Contextual reruns from the tree passed `originalPrompt: undefined`, producing an empty prompt `<<<ORIGINAL_TASK_PROMPT_START>>><<<ORIGINAL_TASK_PROMPT_END>>>`.
  2. `rerunCount` looped `sendPromptNow` with 0ms delay, and because `send_prompt_now` terminates previous agy workers, iteration 2 immediately cancelled iteration 1.
  3. Contextual callbacks passed `repoPath: ""` when `selectedProject` was null.
- **Solution**:
  1. Ensure `buildRerunPrompt` preserves the nudge message and falls back to conversation turn text.
  2. Add inter-iteration delay (or single-shot batching) in `dispatchRerunMessages` to prevent self-cancelling preemption loops.
  3. Enforce 3-tier fallback to resolve `repoPath` from active project, conversation node, or default workspace.

### 2.5 Header Icons Capsule Refinement
- **Defect**: Header navigation icons in `NavMenu.tsx` look mismatched, icon colors lack unity, and the dropdown chevron has an awkward background border inside the pill capsule.
- **Root Cause**: The capsule uses mixed icon colors with harsh dividing borders. The dropdown trigger is styled inconsistently with the icon buttons.
- **Solution**: Wrap into a contiguous segmented pill capsule (`rounded-full`, dark-glass styling `bg-slate-900/80 border border-slate-800`), use harmonic icon tinting, smooth subtle dividers, and clean borderless hover transitions.

### 2.6 Telegram Communication Latency
- **Defect**: Telegram prompt inspection (`/active`, `/tree`, `/seek`) and prompt sending (`/send`, `/rerun`) are extremely slow.
- **Root Cause**:
  1. `daemon.rs` enforces a 60-second startup sleep.
  2. `dispatch_send_prompt` and `rerun_prompt_by_id` synchronously await `capture_and_send_proof_shot`, which executes PowerShell scripts, thread sleeps (including a 4-second `std::thread::sleep`), and photo uploads before acknowledging the command (3 to 12+ seconds).
  3. Missing `sendChatAction` typing indicators and delayed `answerCallbackQuery`.
  4. Redundant process creation storm (`gitmap.exe` spawned 15–25 times sequentially in `cluster.rs` / `node_label.rs` due to un-cached cluster queries).
- **Solution**:
  1. Remove the 60s startup sleep; start polling immediately on daemon launch.
  2. Acknowledge prompt dispatch immediately (<100ms) with a text confirmation; spawn screenshot proof capture as a detached background task.
  3. Send immediate `typing` action and fast callback acknowledgement.
  4. Add an in-memory cache with 30s TTL for `query_gitmap_cluster_nodes` to eliminate sequential process spawns.

---

## 3. Scope & Non-Negotiable Invariants
- **Preserve Send & Enqueue**: The side-by-side segmented capsule `[ Send (Preempt) | Enqueue (FIFO) | v ]` in `DetailHeader.tsx` must remain completely intact and functional.
- **Relative Git Paths**: All documentation, plans, code references, and release notes must strictly use repository-relative paths.
- **Zero Raw Git Commits**: Atomic commits via GitMap (`gitmap cpb "<module> - <summary>"`).
- **Interactive HTML Verification First**: Provide a visual HTML prototype for user inspection and feedback before proceeding to production code commits.

# Master Plan: IDE Crash Recovery, PID Focus & Recent Prompt Auto-Resume

## Status: Completed

## User Request (Verbatim)

```text
https://prnt.sc/qcEYl_BnoDhh

In some cases, the IDE gets crashed. So if the AGM tool or Antigravity Manager is running, then let's say after every two minutes, it will check if the Antigravity crashed. So if the Antigravity is crashed or not in the focus, it will try to put it to the focus. So because it would know the PID. Now, in any case, it would see Antigravity is not running. Then it would automatically click on the fast-forward button to find the newest, let's say, account, and reopen the Antigravity. And during this process, every time we reopen using the fast-forward button, we need to include one more option there. The option should actually understand, like in last few cases, how many prompts and projects were running with the prompts. So let's say a project was running. Let's say five projects were running with the prompts, and we can understand by the time. If you look into the time, like last time when it was running. If it was running more than one day or more than five hours ago, or let's say one, two hours ago, then we can skip. But if it is like in minutes, let's say less than one hour, a project is running. So what it would do is that the last prompt that it has executed with the image, it would take that and send it directly to that project to execute immediately. That would actually spin up the boot process. Do you understand? Can you please help me with this? If you don't understand, you cannot do it. So you can ask me questions if you have any ambiguity. Is it clear?
```

---

## Visual Specification Reference

![Screenshot](assets/screenshots/ide-crash-recovery-and-prompt-resume-01.png)

The screenshot shows an unresponsive / blank desktop where an Antigravity IDE instance was running or taskbar icon clicked but failed to focus or quietly crashed in the background.

---

## Extracted Actionable Task List

- [x] Task-01: Ingest screenshot from `https://prnt.sc/qcEYl_BnoDhh` into `assets/screenshots/ide-crash-recovery-and-prompt-resume-01.png` and record verbatim prompt.
- [x] Task-02: Enhance the 2-minute IDE crash and focus watchdog in `src-tauri/src/modules/process.rs` and `src-tauri/src/modules/auto_switcher.rs` to track all instance PIDs, use Win32 `ShowWindow(SW_RESTORE)` / `SetForegroundWindow` fallback, and ensure the watchdog operates when `auto_focus_window` is active.
- [x] Task-03: Enhance Fast-Forward rotation in `src/stores/useInstanceStore.ts`, `src/components/navbar/InstanceSelector.tsx`, and `src/pages/Instances.tsx` to return and display resumed prompt counts and refill runway in UI toasts.
- [x] Task-04: Expose configurable recency threshold options (30m, 1h, 2h, 5h, 1d) and auto-resume toggle in `src/components/settings/AutoSwitcherSettings.tsx`.
- [x] Task-05: Verify and consolidate lean subtasks into `.ai-memory/plans/completed/03-ide-crash-recovery-and-prompt-resume.md` and update index.

---

## Architectural Context & Subtask Consolidation

### Subtask 01: Watchdog Process Focus & Independent Execution
- **Target Files:**
  - `src-tauri/src/modules/process.rs`
  - `src-tauri/src/modules/auto_switcher.rs`
- **Implementation:**
  - In `src-tauri/src/modules/process.rs`, created `focus_instance_pids(&[u32])` which iterates over all PIDs in the instance process tree. It first attempts `WScript.Shell.AppActivate(pid)`. If that does not bring a window to the front, it executes a Win32 P/Invoke fallback finding the first PID with a valid `MainWindowHandle`, calling `ShowWindow(SW_RESTORE=9)` and `SetForegroundWindow(MainWindowHandle)`.
  - In `src-tauri/src/modules/auto_switcher.rs`, decoupled `check_and_recover_crashed_instance` gating: `let is_watchdog_active = switcher_cfg.is_enabled || switcher_cfg.auto_focus_window;`. If `!is_watchdog_active`, returns early. Otherwise, checks process status every 120s and passes all instance PIDs to `focus_instance_pids`. If dead, cleans lockfiles and triggers fast-forward recovery.

### Subtask 02: Fast-Forward Prompt Resume UI Feedback
- **Target Files:**
  - `src/stores/useInstanceStore.ts`
  - `src/components/navbar/InstanceSelector.tsx`
  - `src/pages/Instances.tsx`
- **Implementation:**
  - In `src/stores/useInstanceStore.ts`, updated `smartRotateProfileAccount` return signature to provide `resumedProjectsCount` and `skippedProjectsCount` alongside `accountEmail`, `instanceName`, and `daysUntilRefill`.
  - In `src/components/navbar/InstanceSelector.tsx` and `src/pages/Instances.tsx`, updated the smart rotate button toast message to display ` · Auto-resumed {count} active project(s) (<1h)` whenever active projects are recovered.

### Subtask 03: Watchdog & Recency Settings UI
- **Target Files:**
  - `src/components/settings/AutoSwitcherSettings.tsx`
- **Implementation:**
  - Added a Recency Cutoff Window selector dropdown offering 30 Minutes (`1800`), 1 Hour [Recommended] (`3600`), 2 Hours (`7200`), 5 Hours (`18000`), and 1 Day (`86400`).
  - Added descriptive guidance text explaining that projects run within this window are auto-resumed with prompt and image attachments to spin up the clean boot process, while older inactive projects are safely skipped.

---

## Domain Guidelines Compliance

1. **Rule 1 (Boolean Principles):** All boolean expressions use implicit checks; zero `== true` evaluations; zero mixed polarity (`&& !`).
2. **Rule 4 (Strict Lowercase):** All newly added files and assets use lowercase names.
3. **Rule 5 (Relative Git Paths):** All asset, documentation, and source references use strictly relative git paths.
4. **Rule 6 (AppError Standards):** Rust and Go error handling adhere to structured AppError patterns.
5. **Workflow Protection:** Zero edits made to `.github/workflows/*.yml`.

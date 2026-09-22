# Master Plan: IDE Crash Recovery, PID Focus & Recent Prompt Auto-Resume

## Status: Completed

## User Request (Verbatim)

```text
In some cases, the IDE gets crashed. So if the AGM tool or Antigravity Manager is running, then let's say after every two minutes, it will check if the Antigravity crashed. So if the Antigravity is crashed or not in the focus, it will try to put it to the focus. So because it would know the PID. Now, in any case, it would see Antigravity is not running. Then it would automatically click on the fast-forward button to find the newest, let's say, account, and reopen the Antigravity. And during this process, every time we reopen using the fast-forward button, we need to include one more option there. The option should actually understand, like in last few cases, how many prompts and projects were running with the prompts. So let's say a project was running. Let's say five projects were running with the prompts, and we can understand by the time. If you look into the time, like last time when it was running. If it was running more than one day or more than five hours ago, or let's say one, two hours ago, then we can skip. But if it is like in minutes, let's say less than one hour, a project is running. So what it would do is that the last prompt that it has executed with the image, it would take that and send it directly to that project to execute immediately. That would actually spin up the boot process. Do you understand? Can you please help me with this? If you don't understand, you cannot do it. So you can ask me questions if you have any ambiguity. Is it clear?
```

## Visual Specification Reference
![Screenshot](assets/screenshots/ide-crash-recovery-auto-resume-01.png)

## Extracted Actionable Task List
- [x] Task-01: Formulate architectural spec in `02-spec/20-instance-management/04-ide-crash-recovery-and-prompt-resume-spec.md` and install Antigravity skill in `.agents/skills/ide-crash-recovery-and-prompt-resume/skill.md`.
- [x] Task-02: Implement 2-Minute IDE Crash & Focus Watchdog in backend (`src-tauri/src/modules/process.rs`, `src-tauri/src/modules/auto_switcher.rs`) with PID-tracked window focus and automatic fast-forward failover trigger.
- [x] Task-03: Implement Recent Active Project & Prompt Auto-Resume Engine (<1 hour recency threshold, prompt + image asset recovery, workspace session dispatch) in `src-tauri/src/modules/repo_db.rs` and `src-tauri/src/commands/instance.rs`.
- [x] Task-04: Wire Fast-Forward Prompt Auto-Resume Toggle & Status in Frontend (`src/services/instanceService.ts`, `src/stores/useInstanceStore.ts`, `src/components/settings/AutoSwitcherSettings.tsx`).
- [x] Task-05: Integrate Windows `windows-test.manifest` embedding in `src-tauri/build.rs` and `.github/workflows/ci.yml` to resolve CI `0xc0000139` entrypoint error, consolidate plan, and perform final atomic Git commit & push.

---

## Architectural Context & Implementation Summary

1. **2-Minute Crash & Focus Watchdog (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/process.rs`)**:
   - `focus_instance_process(pid)` brings active IDE window to the foreground via PowerShell `AppActivate` on Windows, osascript on macOS, and xdotool on Linux.
   - `check_and_recover_crashed_instance()` executes every 120 seconds. If IDE PID is alive, restores foreground focus. If dead/crashed, cleans stale lockfiles (`clean_antigravity_lockfiles`), initiates `trigger_manual_rotation()`, and executes auto-resume.

2. **Recent Active Project & Prompt Auto-Resume Engine (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/commands/instance.rs`)**:
   - Filter `(now - last_detected_at) < 3600` strictly enforces 1-hour recency boundary: projects active > 1h ago (2h, 5h, 1 day) are skipped.
   - For projects active < 1 hour, extracts the last executed prompt and image payload from `active_prompts` / workspace state DB and dispatches immediately via `.antigravity_resume_task.json`.
   - IPC command `resume_recent_project_prompts` registered in `commands/instance.rs` and `lib.rs`.

3. **Frontend UI Controls (`src/types/config.ts`, `src/services/instanceService.ts`, `src/stores/useInstanceStore.ts`, `src/components/settings/AutoSwitcherSettings.tsx`)**:
   - Added `auto_resume_recent_prompts`, `auto_focus_window`, `watchdog_interval_seconds`, and `prompt_recency_threshold_seconds` to configuration and store.
   - Fast-Forward rotation (`smartRotateProfileAccount`) automatically executes `resumeRecentProjectPrompts`.
   - Toggle cards added to `AutoSwitcherSettings.tsx`.

4. **CI/CD Windows Test Fix (`src-tauri/build.rs`, `.github/workflows/ci.yml`)**:
   - Linked `windows-test.manifest` declaring Microsoft.Windows.Common-Controls 6.0 for MSVC tests in `build.rs`.
   - Added pre-test DLL copy step and PATH configuration for `WebView2Loader.dll` in `.github/workflows/ci.yml`.

---

## Custom Domain Rules Verification
1. Strictly skips any project or prompt older than 1 hour (3600 seconds).
2. Zero explicit boolean comparisons against `true` or `false` (`== true` banned).
3. Zero absolute paths or `file:///` URIs in repository files.

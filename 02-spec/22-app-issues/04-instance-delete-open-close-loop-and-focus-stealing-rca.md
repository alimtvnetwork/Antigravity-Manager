# Issue 04: Instance Delete Error Trace, Open/Close Restart Loop, and Unwanted Background Focus Stealing RCA

- **Severity:** High
- **Status:** Fixed
- **Related Spec:** [25-instance-delete-auto-switcher-navbar-and-plaintext-email-fixes.md](../21-app/25-instance-delete-auto-switcher-navbar-and-plaintext-email-fixes.md)

---

## 1. Reproduction

1. **Instance Delete Failure:**
   - Open the `INSTANCES / PROFILES` dropdown or `/instances` page where `#2 Default Copy` is `ACTIVE` or running.
   - Click the Trash (`Delete`) button on `#2 Default Copy`.
   - The deletion throws an OS error trace (`The process cannot access the file because it is being used by another process` or fails to delete when `is_active` is set).
2. **Instance Open/Close Restart Loop:**
   - Create or launch a new instance profile.
   - Because the Electron/Chromium launcher process forks a window child process and exits its parent PID within 2 seconds, the background crash watchdog (`auto_switcher.rs`) checks only the initial parent `pid`, concludes the instance crashed, and repeatedly kills and relaunches it in an endless open/close loop.
3. **Unwanted Window Focus Stealing:**
   - While working in Windows Explorer or another application, background timers or window-restore focus callbacks (`force_restore_and_focus_win32` / `auto_focus_window`) invoke Win32 `SetForegroundWindow` and `SwitchToThisWindow`, abruptly hijacking window focus back to Antigravity IDE.

---

## 2. Cause (Root Cause Analysis)

1. **`delete_instance` Lack of Process Termination & File-Lock Resilience:**
   - `delete_instance` in `src-tauri/src/modules/instance.rs` called `std::fs::remove_dir_all(&dir)` directly without first terminating any active processes holding `--user-data-dir=<dir>`. On Windows, Chromium locks `state.vscdb`, `Cookies`, and `lockfile`, causing `remove_dir_all` to return `Err` and abort before updating `instances.json`.
2. **PID Tracking Mismatch in `refresh_running_statuses` & Crash Watchdog:**
   - `refresh_running_statuses` only checked `system.process(Pid::from_u32(pid))` for the initial wrapper PID. When `Antigravity.exe` handed off execution to a child/worker process with `--user-data-dir=<instance_dir>`, the wrapper PID exited, causing `is_running` to flip to `false` and triggering the crash watchdog to re-spawn `launch_instance` in a loop.
3. **Aggressive Win32 Foreground Lock Bypass in Background Loops:**
   - `auto_focus_window` defaulted to `true`, and background tasks triggered window focus recovery (`force_restore_and_focus_win32`) even when the user was actively interacting with external applications like Windows Explorer.

---

## 3. Fix

1. **Graceful Stop + Resilient Directory Purge in `delete_instance`:**
   - `delete_instance` now explicitly invokes `let _ = close_instance(&instance_id);` first, waits briefly for handles to release, resets `active_instance_id` to `"default"` if the deleted profile was active, removes the profile from `instances.json` and `instances.db` unconditionally, and cleans `<instance_dir>` resiliently without failing the IPC call.
2. **Command-Line `--user-data-dir` Process Discovery & Loop Prevention:**
   - `refresh_running_statuses` now scans live processes for `--user-data-dir=<instance_dir>` if the initial `pid` exited, seamlessly adopting the live window PID.
   - The background loop in `auto_switcher.rs` no longer forcefully kills/relaunches instances unless an explicit quota rotation or user action occurs.
3. **Disable Unsolicited Background Focus Hijacking:**
   - Default `auto_focus_window` to `false` across backend and frontend, and restrict `force_restore_and_focus_win32` strictly to explicit user-initiated tray/window restore actions.

---

## 4. Prevention

- All instance lifecycle operations (`create`, `launch`, `close`, `delete`) are verified end-to-end via CLI commands (`--create-profile`, `--list-profiles`, `--delete-profile`) and guarded against OS file-lock errors.

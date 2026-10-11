# Architecture Specification: Profile Switch Crash & Complete Process Termination

## 1. System Invariants & Problem Taxonomy

### 1.1 Invariant: Absolute Process Termination on App Close
When the user requests to close Antigravity Manager (via the window close `[X]` button, Alt+F4, system tray "Quit", or custom titlebar / navigation buttons), the operating system process MUST terminate completely and immediately. Zero orphaned processes, background polling threads, or listening network sockets may linger in Windows Task Manager.

**Root Cause of Task Manager Residual Processes:**
1. `src-tauri/src/lib/setup_app.rs`: Lines 256–289 intercept `tauri::WindowEvent::CloseRequested`. If `tray_enabled` is true (the default), `api.prevent_close()` is unconditionally invoked, and `window.hide()` is called. The process is never closed; it simply runs invisibly.
2. `src-tauri/src/lib/run.rs`: Lines 396–405 intercept `tauri::RunEvent::ExitRequested`. If `tray_enabled` is true, `api.prevent_exit()` is unconditionally called. Tauri is explicitly instructed NEVER to exit.
3. `src-tauri/src/commands/cmd_window.rs`: `close_window` delegates to `window.close()`, which triggers `CloseRequested`, which hits `prevent_close()`, masking the exit request.

**Remediation Architecture:**
Implement a centralized, graceful, and definitive exit routine: `crate::modules::process::exit_app_fully(&AppHandle)`:
- Shuts down the Cloudflared Zero Trust tunnel subprocess with bounded timeout (500ms).
- Gracefully terminates the Admin Server and releases TCP listening ports (500ms).
- Shuts down proxy instances and token manager (400ms).
- Calls `std::process::exit(0)` to force absolute process termination from Windows Task Manager.
- Wire this function directly into `WindowEvent::CloseRequested`, `RunEvent::ExitRequested`, and `cmd_window::close_window`.

---

### 1.2 Invariant: Profile Switch Crash Resilience & Process Spawn Fallback Ladder
Switching an account into an instance profile involves five transactional stages:
1. `[Stage 1/5]` Backup active in-flight prompts for the target instance.
2. `[Stage 2/5]` Terminate running instance process (`close_instance_verified`).
3. `[Stage 3/5]` Inject fresh OAuth token and machine credentials into `state.vscdb` and `storage.json`.
4. `[Stage 4/5]` Update instance registry and write-through cache to bind the new account.
5. `[Stage 5/5]` Relaunch the instance IDE process preserving workspace folders.

**Root Cause of Switch Failure & "os error 5: Access is denied":**
1. Cloned profile executables (`Antigravity-<id>.exe`) frequently encounter Windows file access restrictions, Defender quarantine, or missing companion Electron DLLs when launched.
2. If `spawn_with_job_breakaway_fallback` fails with `os error 5: Access is denied`, the fallback in `src-tauri/src/modules/instance/launch.rs:566-602` used `Command::new("cmd.exe")` without job breakaway fallback, which itself failed with `os error 5` when running within a restricted Windows Job Object.
3. Furthermore, the cmd syntax `/c start "" <target_bin>` suffered from argument strip defects, causing `shell_cmd.status()` to fail.
4. When launch failed in `switch.rs:312`, the error was propagated via `?`, failing the entire account switch transaction even though credentials had already been successfully injected into `state.vscdb`!
5. In `src/services/instanceService/lifecycle.ts:199`, `openErrorModal` was called, presenting a blocking error modal over the user interface.

**Remediation Architecture:**
1. Multi-Stage Windows Launch Ladder in `src-tauri/src/modules/instance/launch.rs`:
   - Stage 1: Target executable with `spawn_with_job_breakaway_fallback`.
   - Stage 2: Detected base binary (`Antigravity.exe`) with `spawn_with_job_breakaway_fallback`.
   - Stage 3: Windows Shell dispatch via `cmd.exe /c start "" ...` with `spawn_with_job_breakaway_fallback`.
   - Stage 4: PowerShell `Start-Process` (`powershell.exe -NoProfile -WindowStyle Hidden -Command Start-Process -FilePath ... -ArgumentList ...`), which invokes `ShellExecuteEx` and guarantees job object breakaway.
2. Switch Transaction Non-Fatal Auto-Relaunch:
   - In `switch.rs:294-314`, if auto-relaunch fails after successful credential injection and registry binding, log a warning and return `Ok(())`. Do NOT abort or roll back the successful switch.
3. Frontend Defensive Error Handling:
   - In `SwitchAccountDialog.tsx`, catch any errors gracefully with toast notifications instead of popping an error modal.

---

### 1.3 Invariant: Rich Telegram Card Formatting, Custom Copy Buttons & Fleet Visibility
1. Telegram Notification Formatting:
   - Use HTML parse mode (`parse_mode: "HTML"`).
   - Format cards with visual sections: `<b>⚡ Antigravity Manager: Account Switched</b>`, clean status rows, and prompt code blocks (`<pre><code>...</code></pre>`).
   - Add Telegram Bot API inline keyboard button with `copy_text` (`{"inline_keyboard": [[{"text": "📋 Copy Prompt", "copy_text": {"text": prompt}}]]}`).
2. Standalone Fleet & Machine Identity without Supabase:
   - Embed machine hostname, LAN IP, instance name, and active account email in notification payloads.
   - Add `/fleet`, `/instances`, `/accounts`, and `/whoami` to `src-tauri/src/modules/telegram_inbound/processing.rs` so users can inspect multi-machine setups directly via Telegram without Supabase.

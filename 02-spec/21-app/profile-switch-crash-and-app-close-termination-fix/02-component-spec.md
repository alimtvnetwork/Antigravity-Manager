# Component Specification: Profile Switch Crash & Complete Process Termination

## 1. Subsystem Components & Responsibilities

### 1.1 Process Termination Subsystem
- **Module**: `src-tauri/src/modules/process/lifecycle.rs` (or `exit.rs` / `process.rs`)
  - Function: `pub fn exit_app_fully(app_handle: &tauri::AppHandle)`
  - Sequence:
    1. Log shutdown intent at `info` level.
    2. Block-on async cleanup with bounded timeouts:
       - Cloudflared tunnel: 500ms timeout.
       - Admin server TCP listener: 500ms timeout.
       - Proxy service instances & TokenManager: 400ms timeout.
    3. Unconditionally invoke `std::process::exit(0)`.
- **Event Listeners**:
  - `src-tauri/src/lib/setup_app.rs`:
    `handle_window_event` on `WindowEvent::CloseRequested` calls `crate::modules::process::exit_app_fully(window.app_handle())`.
  - `src-tauri/src/lib/run.rs`:
    `RunEvent::ExitRequested` calls `crate::modules::process::exit_app_fully(app_handle)`.
  - `src-tauri/src/commands/cmd_window.rs`:
    `close_window` calls `crate::modules::process::exit_app_fully(&window.app_handle())`.

### 1.2 Process Launch & Switch Resilience
- **Module**: `src-tauri/src/modules/instance/launch.rs`
  - Function: `launch_instance_with_workspaces`
  - 4-Stage Fallback Ladder:
    - Stage 1: Direct target executable spawn (`spawn_with_job_breakaway_fallback`).
    - Stage 2: Detected base binary fallback (`detect_antigravity_with_diagnostics` + `spawn_with_job_breakaway_fallback`).
    - Stage 3: Windows Shell dispatch (`cmd.exe /s /c "start \"\" \"<base_exe>\" ..."`).
    - Stage 4: PowerShell `Start-Process` (`powershell.exe -NoProfile -WindowStyle Hidden -Command "Start-Process ..."`).
- **Module**: `src-tauri/src/modules/instance/switch.rs`
  - Function: `switch_account_to_instance`
  - Resilient Stage 5 relaunch: If launch fails after successful credential injection and registry update, log warning and return `Ok(())`.
- **Frontend Dialog**:
  - `src/pages/instances/SwitchAccountDialog.tsx`:
    Defensive try/catch; close modal on success and display toast.

### 1.3 Telegram Chat Rich Card & Fleet Subsystem
- **Module**: `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
  - Construct HTML message with emoji accents, bold labels, `<pre><code>...</code></pre>` code blocks for prompts.
  - Build `InlineKeyboardMarkup` with `copy_text` button for instant prompt copying.
  - Include Fleet Identity Card (Hostname, LAN IP, Instance ID, Active Account Email).
- **Module**: `src-tauri/src/modules/telegram_inbound/processing.rs`
  - Handle `/fleet`, `/instances`, `/accounts`, and `/whoami` commands.

---
plan: 112-antigravity-ui-instance-prompt-fix
subtask: "008"
title: Diagnose Ubuntu and macOS switching failures with structured logging and error modal integration
domain: fullstack
depends_on:
  - 007-focus-launch-ide-action-and-switch-relaunch.md
citations:
  app_spec: .ai-memory/plans/112-antigravity-ui-instance-prompt-fix.md
  coding_guidelines: AGENTS.md
  strictly_avoid: 02-spec/02-coding-guidelines/01-strictly-avoid.md
target_files:
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/error.rs
  - src-tauri/src/commands/instance.rs
  - src/services/instanceService.ts
  - src/stores/error-store.ts
  - src/lib/error-report-generator.ts
  - src/components/errors/error-modal.tsx
status: pending
---

# 008 — Diagnose Ubuntu & macOS Switching Failures with Structured Logging and Error Modal Integration

## 1. Context & Architectural Root Cause Analysis
Users have observed that on Ubuntu (Linux) and macOS, Antigravity Manager occasionally fails to switch profiles or switch instances properly, while silently failing without displaying actionable diagnostic information. A deep architectural investigation reveals three root causes across the POSIX process lifecycles:

### 1.1. macOS `open -n -a` Transient PID Mismatch
On macOS, `launch_instance_inner_with_extra_workspaces` executes:
```rust
let mut cmd = Command::new("open");
cmd.arg("-n").arg("-a").arg(&exe_str).arg("--args")...;
let child = cmd.spawn()?;
let _ = record_instance_pid(instance_id, child.id(), &data_dir);
```
- **The Bug**: The `/usr/bin/open` utility is a command-line wrapper that sends an Apple Event to `launchd` / `WindowServer` and terminates immediately with exit code 0 in ~20 milliseconds.
- **The Consequence**: `child.id()` is the PID of the short-lived `open` process, **not** the PID of the spawned Antigravity/Electron application! When `record_instance_pid` saves this transient PID into SQLite, subsequent background health checks (`is_instance_running`) immediately see the process as dead. This causes instant status flips, incorrect PID displays, and false "Instance Crashed" states.
- **Remedy**: On macOS, either launch the binary directly inside `.app/Contents/MacOS/` (e.g. `Contents/MacOS/Antigravity` or `Contents/MacOS/Electron`) or query `pgrep -f "Antigravity.*--user-data-dir=.*"` after `open -n -a` to capture the true application PID.

### 1.2. macOS Dynamic Linker (`dyld`) & Entitlement Breakage
When instances use `clone_instance_executable`:
- Electron applications on macOS are strict macOS Application Bundles. They expect frameworks at `@executable_path/../Frameworks/Electron Framework.framework` and resource bundles (`icudtl.dat`, `Contents/Resources`).
- If a binary is copied or symlinked outside its bundle layout, the dynamic linker (`dyld`) crashes immediately with `dyld: Library not loaded`.
- In addition, macOS Gatekeeper / code signing invalidates bundles whose internal structure has been modified.
- **Remedy**: Always maintain the app bundle hierarchy or delegate execution through shell scripts that preserve bundle integrity with exact argument propagation.

### 1.3. Ubuntu / Linux AppImage & Sandbox Collision
- On Ubuntu, Antigravity is frequently deployed via AppImage or Snap. AppImages mount a virtual FUSE filesystem at `/tmp/.mount_AntigrXXXXXX/` and set environment variables: `LD_LIBRARY_PATH`, `APPIMAGE`, `APPDIR`, and `ARGV0`.
- When Antigravity Manager spawns instances with custom `--user-data-dir`, inherited AppImage environment variables cause child processes to attempt loading conflicting libraries, resulting in `GLIBC_X.XX not found`, Wayland/X11 display crashes (`cannot open display`), or Chromium SUID sandbox errors.
- **Silent Stderr Loss**: Code previously set `.stderr(Stdio::null())`. When child processes crashed on startup, all OS error logs and stack traces were completely lost.
- **Remedy**: Capture `stderr` streams, sanitize environment variables, and log full failure details.

## 2. Target Files and Symbols
- **`src-tauri/src/modules/instance.rs`**:
  - Add structured logging `[INSTANCE_SWITCH:*]` across the entire switch and launch lifecycle.
  - Implement robust PID discovery on macOS post-`open` (or direct bundle binary launch).
  - Clean Linux AppImage/FUSE variables and capture `stderr` on process failure.
  - Throw descriptive `AppError::InstanceSwitchFailed` containing OS stderr, tried paths, and environment diagnostics.
- **`src-tauri/src/error.rs`**:
  - Add `AppError::InstanceSwitchFailed` variant with error code `"E7003"`.
- **`src-tauri/src/commands/instance.rs`**:
  - Update `switch_account_to_instance` command to return `Result<(), crate::error::AppError>` instead of `Result<(), String>`, allowing Tauri to serialize the rich error envelope.
- **`src/services/instanceService.ts`**:
  - In `switchAccountToInstance` and `launchInstance`: catch errors, capture with `useErrorStore.getState().captureError(...)`, and immediately call `useErrorStore.getState().openErrorModal(captured)`.
- **`src/lib/error-report-generator.ts`**:
  - Add suggested remediation steps for error code `"E7003"`.

## 3. Detailed Technical Requirements

### 3.1. Structured Backend Logging `[INSTANCE_SWITCH]`
Instrument each phase of `switch_account_to_instance` and `launch_instance`:
1. `[INSTANCE_SWITCH:START]` Target instance ID, target email, previous email.
2. `[INSTANCE_SWITCH:TERMINATE]` Terminating PIDs for instance data directory.
3. `[INSTANCE_SWITCH:INJECT]` Injecting credentials into `state.vscdb`, `storage.json`, and OS keyring.
4. `[INSTANCE_SWITCH:SPAWN]` Spawning IDE process with executable path and CLI arguments.
5. `[INSTANCE_SWITCH:PID_VERIFY]` Verifying that the process is alive and recording confirmed PID.
6. `[INSTANCE_SWITCH:RESTORE]` Dispatching 5-second asynchronous prompt restore.
7. `[INSTANCE_SWITCH:ERROR]` Logging complete OS stderr and diagnostic context upon failure.

### 3.2. Rich Error Model in `src-tauri/src/error.rs`
```rust
#[derive(Error, Debug)]
pub enum AppError {
    // ... existing variants ...

    #[error("Instance switch failed for '{instance_id}': {message}")]
    InstanceSwitchFailed {
        instance_id: String,
        message: String,
        target_account: Option<String>,
        executable_path: Option<String>,
        os_error_details: Option<String>,
        suggested_action: String,
    },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            // ...
            AppError::InstanceSwitchFailed { .. } => "E7003",
            // ...
        }
    }
}
```

### 3.3. Stderr Capture & PID Verification on Linux and macOS
In `src-tauri/src/modules/instance.rs`:
```rust
// On Linux & macOS: capture stderr in a non-blocking pipe or temporary buffer during the first 1000ms
#[cfg(target_os = "macos")]
{
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:SPAWN] macOS: Launching '{}' via open -n -a with args",
        instance_id
    ));
    // Spawn open command
    let mut cmd = Command::new("open");
    cmd.arg("-n").arg("-a").arg(&exe_str).arg("--args");
    // ... args ...
    let status = cmd.status().map_err(|e| {
        AppError::InstanceSwitchFailed {
            instance_id: instance_id.to_string(),
            message: format!("Failed to execute 'open' utility: {}", e),
            target_account: bound_email.clone(),
            executable_path: Some(exe_str.clone()),
            os_error_details: Some(e.to_string()),
            suggested_action: "Verify Antigravity.app permissions in macOS Privacy & Security Settings.".into(),
        }
    })?;

    if !status.success() {
        return Err(AppError::InstanceSwitchFailed {
            instance_id: instance_id.to_string(),
            message: format!("'open' exited with code {:?}", status.code()),
            target_account: bound_email.clone(),
            executable_path: Some(exe_str.clone()),
            os_error_details: Some(format!("Exit code: {:?}", status.code())),
            suggested_action: "Check if another Antigravity instance is locked or corrupted.".into(),
        });
    }

    // Verify true application PID post-spawn
    std::thread::sleep(std::time::Duration::from_millis(500));
    let real_pids = find_pids_for_data_dir(&data_dir, true);
    if let Some(&real_pid) = real_pids.first() {
        crate::modules::logger::log_info(&format!(
            "[INSTANCE_SWITCH:PID_VERIFY] Confirmed real macOS IDE PID {} for instance '{}'",
            real_pid, instance_id
        ));
        let _ = record_instance_pid(instance_id, real_pid, &data_dir);
    } else {
        crate::modules::logger::log_warn(&format!(
            "[INSTANCE_SWITCH:PID_VERIFY] Could not locate running PID for data_dir '{}' after launch",
            data_dir
        ));
    }
}
```

### 3.4. Frontend Error Store & Error Modal Trigger
In `src/services/instanceService.ts`:
```typescript
export async function switchAccountToInstance(accountId: string, instanceId?: string): Promise<void> {
    try {
        return await invoke('switch_account_to_instance', { accountId, instanceId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.switchAccountToInstance',
            endpoint: 'switch_account_to_instance',
            triggerAction: 'switch_account_to_instance',
            targetId: instanceId || 'default',
            context: { accountId, instanceId },
        });
        // Automatically pop open ErrorModal with comprehensive diagnostic details
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function launchInstance(instanceId: string): Promise<void> {
    try {
        return await invoke('launch_instance', { instanceId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.launchInstance',
            endpoint: 'launch_instance',
            triggerAction: 'launch_instance',
            targetId: instanceId,
        });
        // Automatically pop open ErrorModal with comprehensive diagnostic details
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}
```

### 3.5. Remediation Advice in `src/lib/error-report-generator.ts`
Add diagnostic recommendations for `E7003`:
```typescript
case 'E7003':
    fixes.push(
        'Check OS process permissions: On Ubuntu, ensure the AppImage has execute permissions and FUSE is available (`sudo apt install libfuse2`).',
        'On macOS, check System Settings > Privacy & Security > Accessibility to ensure Antigravity Manager can launch and focus applications.',
        'Verify that stale zombie instances are not holding file locks on `state.vscdb` or `code.lock` in the instance data directory.',
        'Try using "Focus IDE" or launching the instance manually to inspect terminal output.'
    );
    break;
```

## 4. Constraints & Non-Negotiables
- Throw typed `AppError` and preserve full OS stderr details rather than silencing errors.
- Never record transient launcher PIDs (such as `/usr/bin/open`) as the long-lived instance PID.
- Both Ubuntu and macOS paths must be thoroughly covered.
- Frontend must automatically display `ErrorModal` when switching fails so the user can copy full reports and inspect suggested fixes.

## 5. Verification & Acceptance Tests
1. **macOS PID Discovery**: Verify that on macOS, the PID recorded for an instance is the actual IDE process PID and not the `open` wrapper PID.
2. **Error Modal Trigger**: Simulate a launch failure (e.g. invalid executable path). Verify that `ErrorModal` opens automatically with error code `E7003`, OS error details, and suggested remediation steps.
3. **Structured Logs Check**: Trigger an account switch and inspect terminal/log bridge logs. Confirm `[INSTANCE_SWITCH:START]`, `[INSTANCE_SWITCH:TERMINATE]`, `[INSTANCE_SWITCH:INJECT]`, `[INSTANCE_SWITCH:SPAWN]`, and `[INSTANCE_SWITCH:PID_VERIFY]` messages appear sequentially.
4. **Pre-flight Checks**:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`

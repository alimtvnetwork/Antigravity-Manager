# Root Cause Analysis: Instance Launch Failure & Same User Token Isolation

## 1. Symptom

1. When switching accounts to an instance profile (`Default Copy Copy`) or launching an instance from the navbar/dropdown, Antigravity either failed to open a window, or opened with the previous user account rather than the selected account.
2. In the Windows taskbar, an Antigravity icon was present, but no main window was displayed or interactive.
3. The installer script `install.ps1` did not uninstall the pre-existing `lbjlaq/Antigravity-Manager` application, causing the Windows Taskbar to launch the legacy binary from `AppData\Local\Antigravity Tools` instead of the newly installed release.

## 2. Root Cause

1. **Keyring Token Desynchronization (Antigravity >= 2.0.0):** Antigravity 2.15.0 reads user authentication credentials from the operating system credential manager / keychain (`gemini:antigravity`). `switch_account_to_instance` was only injecting tokens into SQLite `state.vscdb` (which is legacy and ignored by Antigravity >= 2.0.0), while `write_to_system_keyring` was never invoked. As a result, both instances loaded the identical user credentials from the system keyring.
2. **Electron Single-Instance Lock Collision:** Antigravity enforces `app.requestSingleInstanceLock()`. When launching a new profile while another Antigravity process was active, the new process detected an existing lock and terminated immediately in under 50ms with code 0.
3. **GUI Window Suppression via `CREATE_NO_WINDOW` (0x08000000):** `launch_instance` and `start_antigravity` passed creation flag `0x08000000` (`CREATE_NO_WINDOW`) on Windows. This flag is designed for background console processes, but when applied to GUI applications, it suppresses main window creation.
4. **Missing Bound Account Sync on Direct Instance Launch:** `launch_instance` did not read `bound_account_id` or synchronize its credentials to the keyring before spawning the executable.
5. **Installer Missing Legacy Cleanup & Taskbar Pinning:** `install.ps1` and `install.sh` lacked detection for the upstream `lbjlaq/Antigravity-Manager` registry uninstall strings, leaving legacy binaries and taskbar shortcuts intact.

## 3. Resolution

1. **Export and Invoke `write_to_system_keyring`:** Made `write_to_system_keyring` public in `src-tauri/src/modules/integration.rs` and called it in `switch_account_to_instance` and `launch_instance` (for bound accounts) to ensure credentials update before launch.
2. **Process Termination Before Spawn:** Terminate any running Antigravity processes prior to launching a target profile so that Electron's single-instance lock is cleared and the new profile window opens cleanly.
3. **Remove `CREATE_NO_WINDOW`:** Removed `cmd.creation_flags(0x08000000)` from all GUI application spawning logic in `src-tauri/src/modules/instance.rs` and `src-tauri/src/modules/process.rs`.
4. **Upgrade `install.ps1`:** Added `Remove-LegacyUpstreamInstallation` to detect and uninstall `lbjlaq/Antigravity-Manager` via registry and directory scans, and added `Pin-TaskbarShortcut` using both User Pinned Taskbar shortcuts and Windows Shell COM verb pinning for Windows 10, 11, and Windows Server.
5. **Upgrade `install.sh`:** Added `remove_legacy_upstream_installation` for `dpkg`/`rpm` and standalone binaries, and added `pin_ubuntu_dock` via GNOME `gsettings set org.gnome.shell favorite-apps`.

## 4. Prevention & Learnings

1. **Total Ban on `CREATE_NO_WINDOW` for GUI Binaries:** `CREATE_NO_WINDOW` (`0x08000000`) must only be applied to silent background CLI tools (`taskkill`, `git`, `cloudflared`), never to interactive Electron/GUI applications.
2. **Dual-Layer Token Storage:** Every profile or instance credential switch must synchronize both modern Keyring (`gemini:antigravity`) and legacy SQLite (`state.vscdb`) stores.
3. **Automated Legacy Purging in Installers:** Install scripts must always check for and remove pre-existing or upstream fork installations before deploying updated packages.

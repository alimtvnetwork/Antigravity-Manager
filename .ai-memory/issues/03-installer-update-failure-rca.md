# RCA: In-App Installer Update Failure & Version Detection Deadlock

## 1. Symptom

When clicking "Install Now" from the update notification (`/accounts` route) or executing an in-app upgrade in Antigravity-Manager `v4.52.0`, the operation fails with error code `E9001`:
```
Installer update failed: 
```
The error details after the colon were completely blank. Furthermore, if a user requested or historical shell context contained version `4.54` (e.g. `.\install.ps1 -Version "4.54"`), the installer attempted to query and download `https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.54/...` which returned HTTP 404 because versions on GitHub jumped from `v4.52.0` directly to `v4.55.0` (skipping `4.53` and `4.54`).

## 2. Root Cause

1. **Empty Error String**: In `install.ps1`, error messages were printed via `Write-Host`, which outputs exclusively to the Host Information/STDOUT stream. When `install.ps1` failed and exited with code `1`, `tokio::process::Command::output()` in Rust captured an empty `stderr`. Rust returned `Err(format!("Installer update failed: {}", stderr))`, producing the blank `"Installer update failed: "`.
2. **Process Tree Kill & Locked Executable Deadlock**: In `install.ps1`, `Close-ToolProcesses` ran `taskkill.exe /F /T /IM agm-alim.exe`. When launched from within the running application via `run_installer_update`, `agm-alim.exe` was the parent process of `powershell.exe`. Taskkill with `/T` (tree kill) terminated the parent `agm-alim.exe` and instantly killed the child `powershell.exe` before installation could proceed. If `agm-alim.exe` was not killed, the NSIS setup installer failed with access denied because `agm-alim.exe` was running and locked by the Windows kernel.
3. **Unscoped PSReadLine History Pollution**: `Get-InvocationHistoryCandidates` in `install.ps1` read global shell history (`(Get-PSReadLineOption).HistorySavePath`) and matched any URL containing `releases/download/v...` without verifying that the URL belonged to `Antigravity-Manager`. Unrelated downloads (such as `movie-cli-v8/releases/download/v2.336.0/install.ps1`) in the user's history polluted `$Version` with incorrect versions like `v2.336.0`.
4. **Non-Existent 4.54 & Two-Part Version Handling**: Version `4.54` does not exist on GitHub releases. When two-segment versions (`4.54`) were passed, they were not normalized to 3-part SemVer (`4.54.0`), and when GitHub returned 404, the script fell back to obsolete ancient versions (`4.41.0`) instead of the latest verified releases (`4.57.0`, `4.56.0`, `4.55.0`).

## 3. Resolution

1. **Rust Backend (`src-tauri/src/modules/update_checker.rs`)**:
   - In `run_installer_update`, pass `-Update -NoLaunch` to `install.ps1`.
   - If `stderr` is empty when the command fails, extract and report the descriptive failure lines from `stdout` so the error message is never blank.
2. **PowerShell Installer (`install.ps1`)**:
   - In `Write-Err`, write to `[Console]::Error.WriteLine` in addition to `Write-Host` so stderr is populated.
   - In `Get-InvocationHistoryCandidates`, remove reading global PSReadLine history files, and in `Resolve-PinnedVersion`, strictly scope URL regex to `(Antigravity-Manager|agm-alim|antigravity).*releases/download/v...`.
   - Automatically normalize 2-part versions (e.g. `4.54` -> `4.54.0`).
   - In `Close-ToolProcesses`, detect parent PID (`$parentPid`) and preserve the calling application process so in-app update never self-terminates.
   - Before executing the NSIS package, gracefully rename any active `agm-alim.exe` to `agm-alim.exe.old` to bypass Windows running executable file locks.
   - If a requested pinned version does not exist on GitHub releases, replenish `$candidateVersions` with latest verified releases (`4.57.0`, `4.56.0`, `4.55.0`).
3. **Bash Installer (`install.sh`)**:
   - Scope version extraction regex strictly to `Antigravity-Manager`.
   - Normalize two-part versions to SemVer.
   - Update fallback releases array to include verified releases up to `4.57.0`.

## 4. Prevention & Learnings

- **Never Inspect Global Shell History**: Avoid reading persistent shell history files across terminal sessions for contextual parameter detection unless strictly filtered by repository namespace.
- **Robust Subprocess Error Extraction**: Never assume `stderr` contains process failure messages from PowerShell — always fallback to `stdout` lines when capturing process errors in Rust.
- **In-App Executable File Replacement**: On Windows, running executables cannot be deleted or overwritten, but can be renamed within the same directory (`Move-Item agm-alim.exe agm-alim.exe.old`). This allows installers to unpack updated binaries without locking conflicts.

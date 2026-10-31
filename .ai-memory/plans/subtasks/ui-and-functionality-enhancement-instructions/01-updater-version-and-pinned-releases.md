# Subtask 01: Updater Version Detection & Pinned Release Execution

## Scope
- Files: `install.ps1`, `src-tauri/src/modules/update_checker/platform.rs`

## Implementation Steps
1. In `install.ps1`:
   - In `Get-InstalledVersion`, add PE File Header inspection (`[System.Diagnostics.FileVersionInfo]::GetVersionInfo($candidatePath).ProductVersion`) before checking process stdout.
   - Ensure running process binary path and `$PSScriptRoot` are checked before Stage 4 Windows Registry fallback.
   - Invoke `Sync-InstalledAppsRegistryBranding` in `-Update` and `-CheckUpdate` paths to synchronize registry `DisplayVersion` to current version.
2. In `src-tauri/src/modules/update_checker/platform.rs`:
   - Ensure `info.current_version = CURRENT_VERSION.to_string();` takes precedence over script outputs.

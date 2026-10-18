# Subtask 07: Updater Version Detection & Pinned Version Execution

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-09`
Owner: Worker 01

## Objectives
1. Fix `Get-InstalledVersion` in `install.ps1`: inspect running process executables and query PE header `ProductVersion` on disk via `[System.Diagnostics.FileVersionInfo]::GetVersionInfo($exePath).ProductVersion` before registry fallback.
2. Fix `Sync-InstalledAppsRegistryBranding` in `install.ps1`: set `DisplayVersion` to `$currentVer`.
3. Fix `Resolve-PinnedVersion` in `install.ps1`: match GitHub release tag URLs (`/releases/tag/v4.226.3`) so `$isPinned` is correctly `$true`.
4. Allow downgrade when `$isPinned` is `$true` in `install.ps1`.
5. Update `extract_target_version` in `src-tauri/src/bin/agm/update_cmds.rs` to handle tag URLs.

# Subtask 07: App Updater Pinned-Version & v4.190 Trap Fix

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-07`  
**Status:** `READY`

### Objective
Resolve updater version detection and pinned-version execution:
1. In `install.ps1`:
   - Prioritize `agm-ui.exe` as primary GUI executable; copy it to `agm-alim.exe` as well for legacy backward compatibility.
   - Point shortcuts to `agm-ui.exe`.
   - Update `Get-InstalledVersion` to inspect `agm-ui.exe` first, eliminating the stale `v4.190.0` trap.
   - Parse SemVer from GitHub release URLs via regex: `(?:\/tag\/|v|^v)?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?)$`.
2. In `src-tauri/src/modules/delegate_updater/args.rs`:
   - Support release URLs in `parse_delegate_args` so passing a GitHub release tag URL sets `target_version` and enables pinned mode.

### Target Files
- `install.ps1`
- `src-tauri/src/modules/delegate_updater/args.rs`

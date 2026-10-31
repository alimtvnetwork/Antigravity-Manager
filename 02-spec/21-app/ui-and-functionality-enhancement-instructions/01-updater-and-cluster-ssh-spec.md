# Specification: Updater Version Resolution, Pinned Releases & Cluster SSH Parity

## 1. Executive Summary
This specification defines the architectural and implementation requirements for:
1. Eliminating stale version anchors in updater version detection (specifically resolving the `v4.190` anchor bug).
2. Ensuring deterministic execution of pinned releases (e.g. `v4.226.3` / `v4.226.0`).
3. Achieving complete functional parity between Antigravity-Manager's Shared Cluster / SSH Manager and GitMap's SSH multi-node engine on Windows node `W2`.

## 2. Topic 1: Updater Version Detection Ladder

### 2.1 Root Cause & Invariant
- **Root Cause**: `agm-ui.exe` is compiled with Windows GUI subsystem (`windows_subsystem = "windows"`), causing PowerShell stdout capture to yield empty strings. Previous logic fell back to Windows Uninstall Registry `DisplayVersion` (which was frozen at `4.190.0` from a legacy NSIS installer).
- **Core Invariant**:
  - PE File Header inspection (`[System.Diagnostics.FileVersionInfo]::GetVersionInfo($exePath).ProductVersion`) MUST be prioritized over process output capture.
  - Active running process binary and `$PSScriptRoot` paths MUST take precedence over legacy Windows Registry entries.
  - `Sync-InstalledAppsRegistryBranding` MUST execute during `-Update`, `-CheckUpdate`, and delegated updater runs to keep Windows Registry `DisplayVersion` synchronized with the active version.
  - In `src-tauri/src/modules/update_checker/platform.rs`, `info.current_version = CURRENT_VERSION.to_string();` MUST strictly override external script output.

## 3. Topic 2: Pinned Release Execution
- **Root Cause**: Pruned Git tags or non-existent patch releases (e.g. `v4.226.3`) return 404 from GitHub Releases API, and PowerShell pipeline invocations (`irm ... | iex`) drop argument parameters.
- **Specification**:
  - `install.ps1` MUST validate target tags against milestone manifests and provide clean, actionable guidance if an intermediate tag was pruned.
  - Pinned version script runner MUST preserve explicit `-Version '<ver>'` and `-Force` flags through the command pipeline.

## 4. Topic 3: Cluster / SSH Synchronization with GitMap W2 Parity
- **Root Cause**:
  - Antigravity-Manager invoked native Windows OpenSSH with `-o BatchMode=yes`, which forbids password prompts. Node `W2` requires credential authentication or initial public key authorization.
  - GitMap connects via an in-process SSH client with vault password fallback (`rtyrty123@`) and self-heals key authorization via `gitmap ssh fix-auth`.
- **Specification**:
  - In `src-tauri/src/modules/ssh_manager/exec.rs`: When OpenSSH fails with exit code 255 (auth failure), the executor MUST fall back to `gitmap ssh exec <node> "<command>"` or trigger `gitmap ssh fix-auth <node>`.
  - In `src/components/instances/fleet/useFleetMachines.ts`: Local SSH mesh nodes from `ssh_get_nodes` MUST remain visible and queryable even when cloud Supabase sync is disabled.

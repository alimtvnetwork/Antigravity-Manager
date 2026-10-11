# Subtask Implementation Plan: CLI Self-Delegating Updater Downgrade & Runtime Execution Fix (`Worker 02`)

> **Subtask ID:** `02-cli-updater-downgrade-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Parent Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`  
> **Remediation Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/02-rca-and-remediation-spec.md`  
> **Strictly Owned Files (`Worker 02`):**
> - `install.ps1`
> - `src-tauri/src/modules/delegate_updater/args.rs`
> - `src-tauri/src/modules/delegate_updater/run.rs`
> - `src-tauri/src/bin/agm/update_cmds.rs`
> - `src-tauri/src/bin/agm/update_helpers.rs`
> **Status:** `[READY_FOR_WORKER]`  
> **Author:** @aukgit  
> **Assigned Role:** Worker 02 (CLI Updater & PowerShell Installer Specialist)

---

## 1. Context & 4-Part Root Cause Summary (`DEFECT A1–A4` & `DEFECT B1–B5`)

While `Worker 01` remediates Account Switching and Fast-Forward rotation (`DEFECT A1` cross-instance PID contamination in `pid_scan.rs:302`, `DEFECT A2` fallback candidate scoring in `selection.rs:294`, `DEFECT A3` sibling running guard false positives in `switch.rs:59`, and `DEFECT A4` prompt backup migration visibility in `backup_prompts_db/mod.rs`), `Worker 02` owns the end-to-end remediation of the CLI updater (`agm update v4.162` / `agm update 4.162`).

Runtime trace analysis uncovered **five compounding runtime root causes (`DEFECT B1–B5`)** that prevented `agm update v4.162` / `agm update 4.162` from replacing the installed `v4.188.0` binaries even after 2-part SemVer normalization (`4.162` $\to$ `4.162.0`) was added:

### 1.1 DEFECT B1 (`install.ps1:256-257`): Empty `$DesktopDir` Startup Crash
- **Symptom**: `install.ps1` aborts immediately at line 257 with `ParameterBindingValidationException: Cannot bind argument to parameter 'Path' because it is an empty string`.
- **Code Site**: `install.ps1:256-257` calls `$DesktopDir = [Environment]::GetFolderPath('Desktop')` followed immediately by `$DesktopShortcut = Join-Path $DesktopDir "$ShortcutName.lnk"`.
- **Mechanics**: Inside sandboxed Antigravity IDE terminals or headless sessions, `[Environment]::GetFolderPath('Desktop')` returns `""`. With `$ErrorActionPreference = 'Stop'`, `Join-Path "" "$ShortcutName.lnk"` throws a terminating exception before any installer logic runs.
- **Remediation**: Fall back to `Join-Path $RealUserProfile "Desktop"` when `$DesktopDir` is empty/whitespace, and guard `Join-Path` and all shortcut creation/cleanup sites when `$DesktopShortcut` or `$StartMenuShortcut` is `$null`.

### 1.2 DEFECT B2 (`install.ps1:1601-1602` vs `1743`): Uninitialized `$cliDir` Inside Release `try` Block Skips Installer Execution
- **Symptom**: During an update or downgrade (`Test-Path $InstallDir` is `$true`), `install.ps1` downloads the `v4.162.0` release asset, enters the `try` block, and immediately jumps to `catch` at line 1703—completely skipping lines 1619–1624 where the installer/zip is executed!
- **Code Site**: Lines 1601–1602 evaluate `(Join-Path $cliDir "agm.exe")` and `(Join-Path $cliDir "agm-update-cli.exe")`, whereas `$cliDir = Join-Path $env:LOCALAPPDATA "agm-cli"` is not defined until line 1743!
- **Mechanics**: Evaluating `Join-Path $null "agm.exe"` under `$ErrorActionPreference = 'Stop'` throws a terminating exception inside the `try` block at line 1601, aborting the candidate version before lines 1619–1624 can execute `Start-Process` or `Expand-Archive`.
- **Remediation**: Hoist `$cliDir = Join-Path $env:LOCALAPPDATA "agm-cli"` to the top-level path configuration block (~line 255) alongside `$InstallDir`, and remove the late re-declaration at line 1743.

### 1.3 DEFECT B3 (`install.ps1:1711-1725`): Fake Success & False Registry Stamping When Release Installation Fails
- **Symptom**: After `DEFECT B2` causes all `v4.162.0` download/install attempts to fail (`$installedOk = $false`), `install.ps1` prints `"Installation of Antigravity Manager Tools v4.162.0 completed successfully!"`, stamps `4.162.0` into the Windows Uninstall Registry, and exits `0` without replacing `v4.188.0`!
- **Code Site**: `install.ps1:1711-1725` checks `if (-not $installedOk) { if (Test-Path (Join-Path $InstallDir "agm-alim.exe")) { $installedOk = $true } }`.
- **Mechanics**: Because `DEFECT B2` threw at line 1601 before `Remove-Item` at line 1605 could delete the old binary, the previous `v4.188.0` `agm-alim.exe` is still present in `$InstallDir`. Lines 1711–1716 mistake the old binary for a newly installed binary, flip `$installedOk = $true`, overwrite `DisplayVersion` in the Registry with `4.162.0`, and report fake success.
- **Remediation**: When `$isPinned` or `$Update` is active and `-not $installedOk` (and `-not $BuildFromSource`), NEVER fall back to pre-existing binaries in `$InstallDir`. Log a fatal error via `Write-Err` and terminate with `exit 1`.

### 1.4 DEFECT B4 (`src-tauri/src/modules/delegate_updater/args.rs:124-137` & `install.ps1:250, 1743`): Instance Sandbox `USERPROFILE` / `LOCALAPPDATA` Redirection
- **Symptom**: Running `agm update v4.162` inside an Antigravity IDE terminal installs into `<USERPROFILE>/.antigravity_tools/instances/<id>/home/AppData/Local/Programs/agm-alim` and `<USERPROFILE>/.antigravity_tools/instances/<id>/home/AppData/Local/agm-cli` instead of the real host profile (`<REAL_LOCALAPPDATA>/Programs/agm-alim` and `<REAL_LOCALAPPDATA>/agm-cli`).
- **Code Site**: `resolve_default_install_dir()` in `src-tauri/src/modules/delegate_updater/args.rs:124-137` and `$InstallDir` / `$cliDir` in `install.ps1:250, 1743`.
- **Mechanics**: Multi-instance sandboxing overrides `USERPROFILE`, `APPDATA`, and `LOCALAPPDATA` inside IDE instances to point into `.antigravity_tools/instances/<id>/home`. Neither Rust's `delegate_updater` nor `install.ps1` strips the `.antigravity_tools/instances/<id>/home` redirection segment.
- **Remediation**: Strip `.antigravity_tools/instances/<id>/home` from `USERPROFILE` and `LOCALAPPDATA` in both `args.rs` (`strip_instance_sandbox_local_appdata`) and `install.ps1` (`Resolve-HostUserProfile`), ensuring updates always target the real host user profile.

### 1.5 DEFECT B5 (`install.ps1:1619-1624, 1749-1812` & `src-tauri/src/bin/agm/update_helpers.rs:16-20`): NSIS Setup Missing `agm.exe`, Local Build Target Clobbering Pinned CLI Binary, and Hardcoded Path Literals
- **Symptom**: Even when `agm-alim-setup.exe` for `v4.162.0` runs, `agm --version` remains `4.188.0`, and `update_helpers.rs` violates repository relative-path hygiene.
- **Code Site**: `install.ps1:1400-1445, 1619-1624, 1749-1812` and `src-tauri/src/bin/agm/update_helpers.rs:16-20, 86`.
- **Mechanics**:
  1. On GitHub Releases (`v4.162.0`..`v4.162.5`), `agm-alim-setup.exe` (NSIS) only packages `agm-alim.exe` (GUI), whereas `agm-alim_<ver>_windows_x64.zip` contains both `agm-alim.exe` and `agm.exe`.
  2. Lines 1749–1812 of `install.ps1` search `$PSScriptRoot\src-tauri\target\release\agm.exe` before `$InstallDir\agm.exe` even when `$isPinned` is `$true`, overwriting `$cliDir\agm.exe` with the local workspace's `v4.188.0` binary!
  3. `src-tauri/src/bin/agm/update_helpers.rs:16-20` contains hardcoded Windows drive-letter paths in `resolve_gitmap_bin()`.
- **Remediation**:
  1. In `install.ps1`, prefer or additionally extract `agm-alim_<ver>_windows_x64.zip` on Windows release installs so both `agm-alim.exe` and `agm.exe` from the target release are placed into `$InstallDir` and `$cliDir`.
  2. In `install.ps1:1749-1812`, when `$isPinned` is `$true` (or when installing from a release asset without `-BuildFromSource`), NEVER copy `$PSScriptRoot\src-tauri\target\release\agm.exe` into `$cliDir\agm.exe`; copy exclusively from `$InstallDir\agm.exe`.
  3. In `src-tauri/src/bin/agm/update_helpers.rs:16-20`, replace hardcoded drive-letter paths in `resolve_gitmap_bin()` with environment-variable and relative workspace path resolution (`GITMAP_BIN`, `CARGO_HOME`, `USERPROFILE`, `LOCALAPPDATA`, and relative `../gitmap`).

---

## 2. Step-by-Step Implementation Checklist for `Worker 02`

### Step 1: Fix Sandbox Path Resolution in `src-tauri/src/modules/delegate_updater/args.rs` (`DEFECT B4`)
- [ ] **1.1**: Implement `pub fn strip_instance_sandbox_local_appdata(raw_path: &Path) -> PathBuf` in `src-tauri/src/modules/delegate_updater/args.rs`:
  - Normalize path separators (`/` $\to$ `\`) and perform a case-insensitive search for `\.antigravity_tools\instances\`.
  - If found at index `idx > 0`, extract the real host user profile prefix `&normalized[..idx]` and return `PathBuf::from(real_user_root).join("AppData").join("Local")`.
  - Otherwise, return `raw_path.to_path_buf()`.
- [ ] **1.2**: Update `resolve_default_install_dir()` in `src-tauri/src/modules/delegate_updater/args.rs:124-137` to pass `LOCALAPPDATA` (and `USERPROFILE` fallback) through `strip_instance_sandbox_local_appdata` before joining `.join("Programs").join("agm-alim")`.
- [ ] **1.3**: Add a unit test `test_strip_instance_sandbox_local_appdata()` in `args.rs` (using relative/synthetic paths without hardcoded drive letters, e.g., `Path::new("/users/alice/.antigravity_tools/instances/inst-1/home/AppData/Local")`) verifying that the sandbox segment is stripped cleanly.

### Step 2: Ensure Clean Environment & Version Propagation in `src-tauri/src/modules/delegate_updater/run.rs` (`DEFECT B4`)
- [ ] **2.1**: In `src-tauri/src/modules/delegate_updater/run.rs`, ensure `opts.target_version` is normalized (`4.162` $\to$ `4.162.0`) and passed as `-Version "<ver>"` across all PowerShell invocation branches (repo `install.ps1`, cwd `install.ps1`, and remote `install.ps1`).
- [ ] **2.2**: Before spawning `powershell` to run `install.ps1`, ensure `opts.install_dir` is sanitized via `strip_instance_sandbox_local_appdata` if it points inside `.antigravity_tools/instances/`.

### Step 3: Fix `resolve_gitmap_bin()` Relative-Path Hygiene & Parametric `download_release_zip()` in `src-tauri/src/bin/agm/update_helpers.rs` (`DEFECT B5`)
- [ ] **3.1**: In `src-tauri/src/bin/agm/update_helpers.rs:16-20`, refactor `resolve_gitmap_bin()` so it contains **zero** hardcoded drive-letter strings:
  - Check `std::env::var_os("GITMAP_BIN")` first.
  - Check `CARGO_HOME`, `USERPROFILE`, `HOME`, and `LOCALAPPDATA` (stripped of `.antigravity_tools/instances/<id>/home`) joined with `.cargo/bin/gitmap.exe`, `git-work/gitmap/gitmap.exe`, or `work/gitmap/gitmap.exe`.
  - Check relative ancestor paths from `std::env::current_dir()` and `std::env::current_exe()` (e.g., `../gitmap/gitmap.exe`).
  - Fall back to `"gitmap"` on `PATH`.
- [ ] **3.2**: Verify `download_release_zip(dest: &std::path::Path, target_version: Option<&str>)` in `src-tauri/src/bin/agm/update_helpers.rs:80-95` normalizes `target_version` via `crate::update_cmds::normalize_version_str(ver)` and queries `releases/tags/v{clean}` when `Some(ver)` is provided, or `releases/latest` when `None`.

### Step 4: Verify Version Normalization & Direct/Delegated Execution in `src-tauri/src/bin/agm/update_cmds.rs` (`DEFECT B4` & `DEFECT B5`)
- [ ] **4.1**: Verify `normalize_version_str` and `extract_target_version` in `src-tauri/src/bin/agm/update_cmds.rs` normalize all 2-part shorthand inputs (`agm update v4.162`, `agm update 4.162`, `agm update to v4.162`, `agm update --version 4.162`) to `"4.162.0"`.
- [ ] **4.2**: Verify that when `has_explicit_target` is `true`, the `[UP TO DATE]` guard only triggers if `target == current_version` (permitting downgrades when `target != current_version`).
- [ ] **4.3**: Verify that `update_cmds.rs` passes `--install-dir` resolved via the un-sandboxed host `LOCALAPPDATA` when delegating to `agm-update-cli.exe` or `install.ps1`.

### Step 5: Remediate All 5 Runtime Failures in `install.ps1` (`DEFECT B1`, `DEFECT B2`, `DEFECT B3`, `DEFECT B4`, `DEFECT B5`)
- [ ] **5.1 (`DEFECT B4` — Host Profile Un-Sandboxing at Lines 248–255)**:
  - Add `Resolve-HostUserProfile` to strip `\.antigravity_tools\instances\<id>\home` from `$env:USERPROFILE` and `$env:LOCALAPPDATA`.
  - Re-bind `$env:USERPROFILE`, `$env:LOCALAPPDATA` (`Join-Path $RealUserProfile "AppData\Local"`), and `$env:APPDATA` (`Join-Path $RealUserProfile "AppData\Roaming"`).
  - If `$InstallDir` was omitted or points inside `.antigravity_tools\instances\`, set `$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\agm-alim"`.
- [ ] **5.2 (`DEFECT B2` — Hoist `$cliDir` Initialization to Line ~255)**:
  - Initialize `$cliDir = Join-Path $env:LOCALAPPDATA "agm-cli"` right next to `$InstallDir` at line ~255 so `$cliDir` is guaranteed non-null when lines 1601–1602 (`Join-Path $cliDir "agm.exe"`) execute inside the release download `try` block!
  - Remove or guard the duplicate `$cliDir` assignment at line 1743.
- [ ] **5.3 (`DEFECT B1` — Guard Empty `$DesktopDir` at Lines 256–260)**:
  - If `[string]::IsNullOrWhiteSpace($DesktopDir)`, fall back to `Join-Path $RealUserProfile "Desktop"`.
  - Compute `$DesktopShortcut` and `$StartMenuShortcut` conditionally (`if (-not [string]::IsNullOrWhiteSpace($DesktopDir)) { Join-Path $DesktopDir "$ShortcutName.lnk" } else { $null }`).
  - Guard all shortcut creation/inspection sites with `if ($DesktopShortcut -and (Test-Path $DesktopDir))`.
- [ ] **5.4 (`DEFECT B5` — Prefer/Extract `.zip` Release Archive & Protect Pinned CLI Binary at Lines 1400–1445, 1615–1650, 1749–1812)**:
  - In Windows release asset selection (lines 1400–1445) and installation (lines 1615–1650): prefer `agm-alim_<ver>_windows_x64.zip` (or extract `agm.exe` and `agm-alim.exe` from the `.zip` archive into `$InstallDir` and `$cliDir`), because `agm-alim-setup.exe` on `v4.162.x` only packages `agm-alim.exe` (GUI) and omits `agm.exe` (CLI).
  - When extracting a `.zip` archive into `$InstallDir`, handle both flat archives and single-root-folder archives so `agm-alim.exe` and `agm.exe` land directly in `$InstallDir`, and immediately copy `Join-Path $InstallDir "agm.exe"` to `Join-Path $cliDir "agm.exe"`.
  - In lines 1749–1812 (CLI binary installation into `$cliDir`): when `$isPinned` is `$true` (or when `-not $BuildFromSource` and a release package was installed), **never** include `$PSScriptRoot\src-tauri\target\release\agm.exe` in `$cliCandidates`! Only copy `agm.exe` from `$InstallDir\agm.exe` so local workspace builds (`v4.188.0`) cannot clobber the pinned downgrade binary (`v4.162.0`).
- [ ] **5.5 (`DEFECT B3` — Eliminate Fake Success & False Registry Version Stamping at Lines 1711–1735)**:
  - In `install.ps1:1711-1725`, if `-not $installedOk` and `($isPinned -or $Update)` (when `-not $BuildFromSource`), do **not** check `if (Test-Path $existingExe)` and do **not** set `$installedOk = $true`.
  - Instead, emit `Write-Err "Failed to download and install target release v$TargetVersion."` and `exit 1` so failed downgrades never stamp a fake version into the Registry or exit `0`.

---

## 3. Acceptance Criteria for `Worker 02`

- [ ] Running `install.ps1 -Version 4.162 -DryRun` inside a sandboxed Antigravity IDE terminal (where `LOCALAPPDATA` points to `.antigravity_tools/instances/<id>/home/AppData/Local` and `[Environment]::GetFolderPath('Desktop')` is `""`) does NOT throw `ParameterBindingValidationException`, resolves the real host `<LOCALAPPDATA>\Programs\agm-alim` and `<LOCALAPPDATA>\agm-cli`, and resolves target version `v4.162.0`.
- [ ] Running `install.ps1 -Version 4.162` executes the release download `try` block with `$cliDir` already initialized, extracts both `agm-alim.exe` and `agm.exe` (`v4.162.0`) from the release archive into `<LOCALAPPDATA>\Programs\agm-alim` and `<LOCALAPPDATA>\agm-cli`, and does NOT overwrite `<LOCALAPPDATA>\agm-cli\agm.exe` with the local workspace's `src-tauri\target\release\agm.exe`.
- [ ] If all release download candidates fail during `install.ps1 -Version <ver>` or `-Update`, `install.ps1` exits with code `1` and does NOT stamp the failed version into the Registry or print a fake success message.
- [ ] `src-tauri/src/bin/agm/update_helpers.rs` contains zero hardcoded drive-letter paths in `resolve_gitmap_bin()`.

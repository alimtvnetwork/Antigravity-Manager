# Subtask Implementation Plan: CLI Self-Delegating Updater Downgrade & Version Normalization Fix

> **Subtask ID:** `02-cli-updater-downgrade-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Parent Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`  
> **Remediation Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/02-rca-and-remediation-spec.md`  
> **Target Files:**
> - `src-tauri/src/bin/agm/update_cmds.rs`
> - `src-tauri/src/bin/agm/update_helpers.rs`
> - `src-tauri/src/modules/delegate_updater/run.rs`
> - `src-tauri/src/modules/delegate_updater/args.rs`
> - `install.ps1`
> **Status:** `[READY_FOR_WORKER]`  
> **Author:** @aukgit  
> **Assigned Role:** Worker Agent (CLI / DevOps Specialist)

---

## 1. Context & Objectives

Operators running `agm update 4.162` or `agm update to v4.162` encounter complete command failures when attempting to downgrade or pin historical versions:
1. **2-Part SemVer Normalization Gap**: `4.162` and `v4.162` are shorthand versions. GitHub release tags strictly use 3-part SemVer: `v4.162.0`. In `install.ps1:1255`, `Resolve-PinnedVersion` was skipped when `$Version` was truthy, leaving `$cleanPinned` as `"4.162"` and causing 404 download errors.
2. **Hardcoded Query to `releases/latest` in `update_helpers.rs:86`**: `download_release_zip` accepted zero version arguments and unconditionally downloaded `releases/latest`, ignoring the operator's requested release.
3. **Asset Template Guessing in `install.ps1:1423`**: Direct fallback URLs guessed `agm-alim_${cleanPinned}_x64-setup.exe`. In older releases, assets were named `Antigravity-Manager_${cleanPinned}_x64-setup.exe` or used MSI packages.
4. **Premature Downgrade Abort in `install.ps1:1461`**: The updater checked `TargetVersion -le CurrentVersion` and printed *"Already on the latest version"*, blocking intentional downgrades unless `-Force` was explicitly provided.

This subtask plan provides concrete, surgical implementation instructions for Worker agents to resolve all downgrade gaps across the CLI and installer pipeline.

---

## 2. File Boundaries & Invariant Guarantees

Worker agents must restrict edits strictly to the five target files. Unrelated files in `src-tauri/src/proxy/` or frontend UI files must not be touched.

- **Invariant I1 (Deterministic 3-Part Normalization)**: Any version string with 2 numeric segments (e.g. `4.162` or `v4.162`) must be normalized to 3-part SemVer (`4.162.0`) in both Rust CLI handlers and PowerShell installer scripts.
- **Invariant I2 (Version Parameter Propagation)**: When a target version is requested, `download_release_zip` and release queries must target `releases/tags/v{target}`. Calling `releases/latest` is permitted ONLY when no target version was specified.
- **Invariant I3 (Downgrade Gate Bypass for Pinned Versions)**: When `$isPinned` is `$true` in `install.ps1`, the `TargetVersion -le CurrentVersion` check must be bypassed so that intentional downgrades proceed without requiring an explicit `-Force` flag.
- **Invariant I4 (Zero Git Commands)**: Workers must NEVER run raw `git` commands (`git add`, `git commit`, `git status`, `git push`, etc.).
- **Invariant I5 (Zero Search Tools)**: Workers must NEVER execute `grep`, `ripgrep`, `rg`, `Select-String`, or `findstr`. Use `gitmap find` or `view_file`.

---

## 3. Step-by-Step Implementation Instructions

### Step 1: Update CLI Helper Parametric Download in `src-tauri/src/bin/agm/update_helpers.rs`
- In `src-tauri/src/bin/agm/update_helpers.rs`, update `download_release_zip`:
  ```rust
  pub(crate) fn download_release_zip(
      dest: &std::path::Path,
      target_version: Option<&str>,
  ) -> Result<(), String> {
      let client = reqwest::blocking::Client::builder()
          .user_agent("antigravity-manager")
          .build()
          .map_err(|err| err.to_string())?;
      
      let url = if let Some(ver) = target_version {
          let clean = ver.trim().trim_start_matches(['v', 'V']);
          format!(
              "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/v{}",
              clean
          )
      } else {
          "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest".to_string()
      };

      let release: serde_json::Value = client
          .get(&url)
          .header("Accept", "application/vnd.github+json")
          .send()
          .map_err(|err| err.to_string())?
          .error_for_status()
          .map_err(|err| err.to_string())?
          .json()
          .map_err(|err| err.to_string())?;
      ...
  ```
- In `update_cmds.rs:357`, update the call site:
  ```rust
  let target_ver_opt = if has_explicit_target { Some(tag_name.as_str()) } else { None };
  match crate::update_helpers::download_release_zip(&export_path, target_ver_opt) {
  ```
- In `update_helpers.rs:186` (`cmd_update_export_zip`), pass `None` or parse version flags from args.

### Step 2: Propagate Version Flags in `src-tauri/src/bin/agm/update_cmds.rs`
- Ensure `extract_target_version` correctly handles:
  - `to v4.162` -> `"4.162.0"`
  - `revert 4.162` -> `"4.162.0"`
  - `downgrade 4.162` -> `"4.162.0"`
  - `4.162` -> `"4.162.0"`
  - `v4.162` -> `"4.162.0"`
  - `--version 4.162` -> `"4.162.0"`
  - `--version=4.162` -> `"4.162.0"`
- In lines 380–407 (Windows self-delegation block), ensure `tag_name` is passed as `--version <tag_name>` to the isolated delegate updater.

### Step 3: Propagate Version Argument to `install.ps1` in `delegate_updater/run.rs`
- Inspect `src-tauri/src/modules/delegate_updater/run.rs` lines 216–220:
  ```rust
  let ver_arg = opts
      .target_version
      .as_ref()
      .map(|v| format!("-Version \"{}\"", v))
      .unwrap_or_default();
  ```
- Ensure `ver_arg` is passed to PowerShell invocation across all three runner branches (local script, cwd script, and remote fallback script).

### Step 4: Fix 2-Part SemVer Expansion & Downgrade Gate in `install.ps1`
- In `install.ps1` around line 1254, update the pinned version initialization:
  ```powershell
  if ($Version) {
      $clean = ($Version -replace "^v", "").Trim()
      $parts = $clean.Split("-")[0].Split(".")
      if ($parts.Length -eq 2 -and ($parts[0] -match '^\d+$') -and ($parts[1] -match '^\d+$')) {
          $clean = "$clean.0"
      }
      $Version = $clean
      $cleanPinned = $clean
      $candidateVersions.Add($cleanPinned)
      $isPinned = $true
      Write-Step "Respecting pinned release version: v$cleanPinned"
  } else {
      $resolvedPin = Resolve-PinnedVersion -ExplicitVersion $Version -BakedVersion $PinnedVersion
      if ($resolvedPin) {
          $Version = $resolvedPin
          $cleanPinned = ($resolvedPin -replace "^v", "").Trim()
          $candidateVersions.Add($cleanPinned)
          $isPinned = $true
          Write-Step "Respecting pinned release version: v$cleanPinned"
      }
  }
  ```
- In `install.ps1` lines 1457–1464, ensure the downgrade gate respects `$isPinned`:
  ```powershell
  if ($Update -and -not $Force -and -not $isPinned) {
      $curr = Get-InstalledVersion
      if ($curr) {
          try {
              if ((Convert-ToSemVer $TargetVersion) -le (Convert-ToSemVer $curr)) {
                  Write-Success "Already on the latest version ($curr)."
                  return
              }
          } catch {
              if ($curr -eq $TargetVersion) {
                  Write-Success "Already on the latest version ($curr)."
                  return
              }
          }
      }
  }
  ```

---

## 4. Verification & Quality Gates

Worker agents must run the following checks locally:

```pwsh
# 1. Rust Code Formatting
cd src-tauri && cargo fmt -- --check

# 2. Rust Clippy Gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Targeted Unit Test Suite
cd src-tauri && cargo test bin::agm::update_cmds
cd src-tauri && cargo test modules::delegate_updater

# 4. Dry-Run Verification of install.ps1
powershell -NoProfile -ExecutionPolicy Bypass -Command ".\install.ps1 -Version 4.162 -DryRun"

# 5. Pipeline Telemetry Check
gitmap pe -t
```

---

## 5. Acceptance Criteria

- [ ] `powershell .\install.ps1 -Version 4.162 -DryRun` outputs `Target release version: v4.162.0` and attempts to download `v4.162.0` assets (not `v4.162`).
- [ ] Running `install.ps1 -Version 4.162.0` on a machine with `v4.186.0` installed does NOT abort with *"Already on the latest version"*.
- [ ] `download_release_zip` with `Some("4.162.0")` contacts `releases/tags/v4.162.0` and downloads the historical release zip archive.
- [ ] `cargo clippy --all-targets --all-features` completes with zero errors and zero warnings.

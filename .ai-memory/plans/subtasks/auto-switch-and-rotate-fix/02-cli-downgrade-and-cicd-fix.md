# Subtask Plan: CLI Updater Downgrade, Revert Lockout Prevention, and CI/CD Quality Gate Audit

> **Subtask ID:** `02-cli-downgrade-and-cicd-fix`  
> **Parent Plan:** `.ai-memory/plans/auto-switch-and-rotate-fix.md`  
> **Technical Specification:** `02-spec/21-app/auto-switch-and-rotate-fix/02-cli-downgrade-and-cicd-spec.md`  
> **Root Cause Analysis:** `02-spec/22-app-issues/36-auto-switch-fast-forward-downgrade-regression-rca.md`  
> **Status:** `[PENDING]`  
> **Author:** @aukgit  
> **Assigned Role:** Subagent 02 (CLI Updater & CI/CD Gate Specialist)

---

## 1. Objectives & Scope

This subtask implements the technical specifications defined in `02-spec/21-app/auto-switch-and-rotate-fix/02-cli-downgrade-and-cicd-spec.md` to resolve the CLI updater downgrade lockout when operators run:
- `agm update 4.162`
- `agm update to v4.162`
- `agm update revert v4.162`
- `agm update downgrade 4.162`

Additionally, this subtask audits and documents the continuous integration pipeline, distinguishing between the remote GitHub Actions gates (`.github/workflows/ci.yml`) and the 36-job local runner matrix (`03-ai-scripts/06-cicd-local-runner.py` / `gitmap pe -t`).

---

## 2. Non-Negotiable Invariants & Guardrails

1. **Strictly Relative Git Paths**: Use only relative repository paths (e.g., `install.ps1`, `src-tauri/src/bin/agm/update_cmds.rs`). Absolute file paths (`C:\...`, `D:\...`) and `file:///` URIs are strictly forbidden.
2. **Positive Boolean Naming**: All flags, variables, and parameters must use positive polarity (e.g., `isPinned`, `isDowngrade`, `hasCompleted`, `isSandboxed`, `isExplicitTarget`). Negative names like `notPinned`, `noDowngrade`, `disableSandbox` are forbidden.
3. **Zero Git Commands**: Do NOT run `git` commands (`git checkout`, `git commit`, `git push`, etc.) directly via terminal runners.
4. **Pre-flight CI Discipline**: Never run full heavy local test suites or unmocked OS-level commands. Rely on targeted compilation checks (`cargo clippy`, `cargo fmt`, `npm run build`) and remote CI gates.

---

## 3. Step-by-Step Implementation Checklist

### Step 3.1: SemVer 2-Part Normalization in Rust CLI
- [ ] In `src-tauri/src/bin/agm/update_cmds.rs`:
  - Enhance `normalize_version_string(raw: &str) -> String` to handle two-part versions (e.g. `4.162` $\to$ `4.162.0`, `v4.162` $\to$ `4.162.0`, `4.162-beta.1` $\to$ `4.162.0-beta.1`).
  - Update `extract_target_version(args: &[String]) -> Option<String>` to parse positional versions, `to <ver>`, `revert <ver>`, `downgrade <ver>`, `--to <ver>`, `--version <ver>`, and `-v <ver>`.
- [ ] In `src-tauri/src/modules/delegate_updater/args.rs`:
  - Verify that `parse_update_args` invokes `normalize_version_string` across all target version parsing branches.

### Step 3.2: GitHub Tag Resolution Ladder
- [ ] In `src-tauri/src/bin/agm/update_cmds.rs`:
  - Formulate API endpoint targeting canonical tag format: `releases/tags/v{target}`.
  - Implement CDN fallback querying `releases/download/v{target}/updater.json`.
  - Implement releases list fallback iterating over `repos/alimtvnetwork/Antigravity-Manager/releases` when direct tag lookup returns 404.
  - Set boolean flag `is_downgrade = has_explicit_target && !is_up_to_date`.
  - Ensure `should_proceed_update` evaluates to `true` when `has_explicit_target` is active, bypassing standard `is_up_to_date` suppression.

### Step 3.3: PowerShell Installer DesktopDir Null-Safety (Defect B1)
- [ ] In `install.ps1:285-289`:
  - Guard `$DesktopDir = [Environment]::GetFolderPath("Desktop")` against empty or whitespace values.
  - When `$DesktopDir` is empty or whitespace, check `$RealUserProfile` and fall back to `Join-Path $RealUserProfile "Desktop"`.
  - If `$DesktopDir` remains empty, set `$DesktopShortcut = $null` and bypass `Join-Path` to prevent `ParameterBindingValidationException`.
  - Guard all shortcut deletion and creation calls (`Join-Path $DesktopDir ...`) with `if (-not [string]::IsNullOrWhiteSpace($DesktopDir) -and (Test-Path $DesktopDir))`.

### Step 3.4: Early Initialization of `$cliDir` (Defect B2)
- [ ] In `install.ps1`:
  - Ensure `$cliDir = Join-Path $RealLocalAppData "agm-cli"` is placed in the top-level path resolution section (~line 280) alongside `$InstallDir`.
  - Verify that `$cliDir` is initialized prior to entering `Close-ToolProcesses` and the release download loop (~line 1660).
  - Ensure lines 1660–1664 (`$lockCandidates += Join-Path $cliDir "agm.exe"`) never evaluate `Join-Path` against an uninitialized `$null` variable.

### Step 3.5: Anti-Fake Success on Pinned Updates (Defect B3)
- [ ] In `install.ps1:1827-1853`:
  - Verify that when `$isPinned` or `$Update` is active:
    - If `$installedOk` is false, NEVER fall back to existing binaries in `$InstallDir`.
    - Log error via `Write-Err`: `"Failed to download and install target release v$TargetVersion. Existing installation was not modified."`
    - Set `$global:LASTEXITCODE = 1` and terminate with `exit 1`.
  - Prevent `Sync-InstalledAppsRegistryBranding` from executing on failed or aborted updates.
  - Ensure Registry `DisplayVersion` is strictly updated only when the target version has been physically installed and verified on disk.

### Step 3.6: IDE Sandbox Path Unwrapping (Defect B4)
- [ ] In `install.ps1:270-281`:
  - Verify that `Resolve-HostUserProfile` checks if `$RealLocalAppData` or `$RealUserProfile` contains `.antigravity_tools\instances\*\home`.
  - Strip the sandbox prefix to ensure `$RealLocalAppData` targets the real host user profile (`%LOCALAPPDATA%`).
  - Set `$InstallDir = Join-Path $RealLocalAppData "Programs\agm-alim"` and `$cliDir = Join-Path $RealLocalAppData "agm-cli"`.
- [ ] In `src-tauri/src/modules/delegate_updater/args.rs`:
  - Verify `strip_instance_sandbox_local_appdata` and `is_instance_sandbox_path` remove the sandbox directory structure.

### Step 3.7: Companion Zip Fallback for Missing Historical `agm.exe` (Defect B5)
- [ ] In `install.ps1:1680-1695`:
  - When installing Windows release packages, if the downloaded file is `agm-alim-setup.exe` (NSIS), check if `agm.exe` exists in `$InstallDir` after installation.
  - If `agm.exe` is missing, download the companion archive `agm-alim_<ver>_windows_x64.zip` and extract `agm.exe` into `$InstallDir` and `$cliDir`.
- [ ] In `install.ps1:1880-1899`:
  - When `$isPinned` is `$true`, disable fallback copying from local workspace target:
    `Join-Path $PSScriptRoot "src-tauri\target\release\agm.exe"`.
  - Ensure `$cliDir\agm.exe` is strictly sourced from the installed release package.
- [ ] In `src-tauri/src/bin/agm/update_helpers.rs`:
  - Verify `resolve_gitmap_bin()` and `default_update_export_path()` use relative paths and standard environment variables without hardcoded drive letters.

### Step 3.8: CI/CD Quality Gate Audit & Documentation
- [ ] Audit `.github/workflows/ci.yml`:
  - Confirm job definitions: `build-frontend`, `check-rust`, `build-tauri`.
  - Verify that official remote CI is passing across all branches.
- [ ] Audit `03-ai-scripts/06-cicd-local-runner.py` and `03-ai-scripts/02-shared-engine.py`:
  - Document the 36 local quality gates in `CI_JOBS_MATRIX`.
  - Clarify the difference between local polyglot linters and canonical remote CI gates.
  - Document guidelines for interpreting `gitmap pe -t` telemetry.

---

## 4. Verification & Testing Scenarios

| Test ID | Command / Scenario | Verification Criteria | Expected Outcome |
| :--- | :--- | :--- | :--- |
| **TEST-01** | `agm update 4.162` | Argument normalization | Target version resolved as `4.162.0`; queries GitHub tag `v4.162.0` |
| **TEST-02** | `agm update to v4.162` | Keyword syntax parsing | Target version parsed successfully as `4.162.0` |
| **TEST-03** | `agm update 4.162` on modern build | Downgrade suppression bypass | Does not output "Already latest"; initiates downgrade flow |
| **TEST-04** | Headless terminal execution | Desktop directory resolution | DesktopDir safely defaults to `$RealUserProfile\Desktop`; 0 errors |
| **TEST-05** | Update on existing install | `$cliDir` variable scoping | `$cliDir` initialized before lock evaluation; installer executes |
| **TEST-06** | Simulated network failure | Anti-fake success | Script exits with code 1; registry version remains unchanged |
| **TEST-07** | IDE terminal invocation | Sandbox path unwrapping | Binaries installed to host `%LOCALAPPDATA%`, not `.antigravity_tools` |
| **TEST-08** | Downgrade to `v4.162.0` | Companion zip extraction | Both `agm-alim.exe` and `agm.exe` exist and report `4.162.0` |
| **TEST-09** | CI gate verification | GitHub Actions `ci.yml` | All 7 matrix jobs green |

---

## 5. Status & Sign-off

- [x] Specification authored (`02-spec/21-app/auto-switch-and-rotate-fix/02-cli-downgrade-and-cicd-spec.md`)
- [x] Subtask implementation plan authored (`.ai-memory/plans/subtasks/auto-switch-and-rotate-fix/02-cli-downgrade-and-cicd-fix.md`)
- [ ] Root Cause Analysis authored (`02-spec/22-app-issues/36-auto-switch-fast-forward-downgrade-regression-rca.md`)
- [ ] Code modifications applied and validated
- [ ] Pre-flight checks passed

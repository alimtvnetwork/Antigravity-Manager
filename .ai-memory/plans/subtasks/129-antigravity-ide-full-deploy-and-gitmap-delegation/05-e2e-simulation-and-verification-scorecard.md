# Subtask 05: End-to-End Simulation and Verification Scorecard

- **Subtask ID**: `129-subtask-05`
- **Traceability ID**: `Task-06`
- **Spec References**:
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/01-architecture-spec.md`
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/02-component-spec.md`
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/03-root-cause-analysis.md`
- **Target Files**:
  - `scratch/verify_ide_deployment.py`
  - `scripts/verify-ide-deployment.ps1`
- **Status**: Ready for Execution

---

## 1. Objective

Develop an automated, end-to-end verification and simulation harness (`scratch/verify_ide_deployment.py` and `scripts/verify-ide-deployment.ps1`) to audit Antigravity IDE instances and fleet deployments against the formal 7-Gate Scorecard (`VG-01` to `VG-07`). The harness evaluates theme parity, Turbo mode presets, official plugins, builtin skills, extensions paths, and remote delegation readiness without manual intervention.

---

## 2. Granular Implementation Steps

### Step 1: Verification Script Architecture (`scratch/verify_ide_deployment.py`)
- Implement a Python-based diagnostic tool with zero third-party dependencies (using standard library `json`, `os`, `sys`, `pathlib`, `subprocess`):
  - Accepts CLI flags:
    - `--target-id <id>`: Specifies instance ID to audit (e.g. `default`, `gitmap-7418`, or newly cloned test instance).
    - `--data-dir <path>`: Direct path to instance data directory.
    - `--home-dir <path>`: Direct path to instance home directory.
    - `--dry-run`: Emulates checks against staged temporary directory.
    - `--json`: Outputs structured machine-readable JSON scorecard.

### Step 2: Implement the 7 Verification Gates
1. **VG-01: Theme Parity Gate**
   - Parses `<data_dir>/User/settings.json` and `<home_dir>/AppData/Roaming/Antigravity/User/settings.json`.
   - Checks presence and value of `"workbench.colorTheme"`.
   - Asserts non-empty string.
2. **VG-02: Preset Mode Gate**
   - Reads `<data_dir>/User/security_presets.json`.
   - Asserts `"active_preset"` is `"turbo"` or `"eager"`.
   - Checks `settings.json` for `"security.workspace.trust.enabled": false` and `"gemini.experimental.eagerExecution": true`.
3. **VG-03: Plugins Inventory Gate**
   - Scans `<home_dir>/.gemini/config/plugins/`.
   - Asserts existence of:
     - `chrome-devtools-plugin`
     - `data-agent-kit-plugin`
     - `google-antigravity-sdk`
     - `modern-web-guidance-plugin`
4. **VG-04: Skills Completeness Gate**
   - Scans `<home_dir>/.gemini/antigravity/builtin/skills/`.
   - Verifies presence of at least 9 builtin skills with valid `SKILL.md`.
   - Scans `<home_dir>/.gemini/config/plugins/*/skills/` for plugin skills.
5. **VG-05: Extensions Directory Gate**
   - Resolves target extensions directory.
   - Verifies directory exists and contains at least 1 theme extension (e.g. `vscode-theme-onedarker` or equivalent).
6. **VG-06: Remote Delegation Gate**
   - Executes probe command or tests `gitmap cluster run-script localhost scripts/deploy-antigravity-ide-fleet.ps1 -DryRun`.
   - Verifies exit code 0 and proper error trapping.
7. **VG-07: Process Liveness Gate**
   - Verifies existence of `antigravity-ide-keyring-unavailable` bypass marker in data and home directories.
   - Verifies `state.vscdb` integrity via SQLite header inspection.

### Step 3: Terminal Scorecard Formatting
- Formats console output matching the unified ASCII table defined in Section 5 of `02-component-spec.md`.
- Emits clean green `[PASS]` / red `[FAIL]` indicators.
- Returns process exit code `0` if all 7 gates pass, or `1` if any gate fails.

### Step 4: PowerShell Parity Script (`scripts/verify-ide-deployment.ps1`)
- Mirror the verification logic in pure PowerShell for Windows environments where Python may not be installed.

---

## 3. Acceptance Criteria

1. Running `python scratch/verify_ide_deployment.py --target-id default` evaluates all 7 gates on the Default instance.
2. Running the verification script against a newly cloned instance passes all 7 gates without failure.
3. The script returns exit code 0 on complete pass, providing actionable failure messages if any asset is missing.
4. Total execution time of the verification check is under 3 seconds.

---

## 4. Targeted Verification

- Execute verification harness:
  ```powershell
  python scratch/verify_ide_deployment.py --dry-run
  ```
- Inspect terminal scorecard and verify:
  `FINAL OUTCOME: 7/7 GATES PASSED (100% PRODUCTION-READY)`

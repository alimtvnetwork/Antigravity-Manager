# Subtask 03: GitMap Delegation and Remote Machine Automation

- **Subtask ID**: `129-subtask-03`
- **Traceability ID**: `Task-04`
- **Spec References**:
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/01-architecture-spec.md`
  - `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/02-component-spec.md`
- **Target Files**:
  - `src-tauri/src/modules/delegate_updater.rs`
  - `src-tauri/src/commands/instance.rs`
  - `src-tauri/src/lib.rs`
  - `scripts/deploy-antigravity-ide-fleet.ps1`
  - `d:\work\repo-secrets\02-antigravity-manager\scripts\deploy-antigravity-ide-fleet.ps1`
- **Status**: Ready for Execution

---

## 1. Objective

Enable automated, cross-machine remote deployment of the Antigravity IDE fleet using GitMap CLI delegation. Provide a unified execution bridge in Antigravity Manager so developers can trigger remote provisioning on any registered GitMap cluster/SSH node (`w1`, `w2`, `w3`, `u1`, `final-network-machine`) with real-time stdout/stderr streaming, 7-Gate Scorecard validation, and automatic registry enrollment.

---

## 2. Granular Implementation Steps

### Step 1: GitMap Cluster Recipe & Delegation Command Definition
- Define the command contract for delegating the standalone deployment script:
  ```bash
  gitmap cluster run-script <target_node> scripts/deploy-antigravity-ide-fleet.ps1 -Preset turbo -Theme "Default Dark+" -Force
  ```
- Define the high-level fleet broadcast command:
  ```bash
  gitmap agy ssh deploy [target] [--preset turbo] [--theme <theme>] [--except <nodes>]
  ```
- Ensure GitMap node recipe `deploy-antigravity` is recognized by `gitmap cluster node`:
  ```bash
  gitmap cluster node deploy-antigravity <target> [--preset turbo] [--theme <theme>]
  ```

### Step 2: Rust Tauri IPC Bridge Implementation
- In `src-tauri/src/modules/delegate_updater.rs`:
  - Implement helper function `run_gitmap_remote_deploy`:
    ```rust
    pub fn run_gitmap_remote_deploy(
        target_node: &str,
        preset: Option<&str>,
        theme: Option<&str>,
        is_dry_run: bool,
    ) -> Result<GitMapDeployExecution, String>
    ```
  - Spawns `gitmap.exe` with arguments `["cluster", "run-script", target_node, "scripts/deploy-antigravity-ide-fleet.ps1"]`.
  - Appends `-Preset`, `-Theme`, and `-DryRun` flags accordingly.
  - Captures execution stream, calculates elapsed duration, and extracts the 7-Gate Scorecard JSON or ASCII block from stdout.
- In `src-tauri/src/commands/instance.rs`:
  - Expose Tauri command `execute_gitmap_ide_deploy`:
    ```rust
    #[tauri::command]
    pub async fn execute_gitmap_ide_deploy(
        node_alias: String,
        preset: Option<String>,
        theme: Option<String>,
        dry_run: Option<bool>,
    ) -> Result<serde_json::Value, String>
    ```
- In `src-tauri/src/lib.rs`:
  - Register `execute_gitmap_ide_deploy` in the Tauri handler invoke table.

### Step 3: Deployment Payload Packaging & Transmission
- Ensure `deploy-antigravity-ide-fleet.ps1` is bundled in `scripts/` of the Antigravity Manager application and synced with `d:\work\repo-secrets\02-antigravity-manager\scripts\`.
- For remote nodes, verify GitMap utilizes SSH streaming to transfer the script into `%TEMP%\agy-fleet-deploy\` or `/tmp/agy-fleet-deploy/` before remote invocation.

### Step 4: Verification Scorecard Telemetry Extraction
- Parse the terminal output emitted by `Test-AntigravityDeploymentHealth` in the PowerShell script:
  - Detect lines matching `[PASS] VG-01: Theme Parity Gate` through `VG-07`.
  - Compile a structured JSON result:
    ```json
    {
      "node": "w3",
      "success": true,
      "exit_code": 0,
      "gates": {
        "VG-01": true,
        "VG-02": true,
        "VG-03": true,
        "VG-04": true,
        "VG-05": true,
        "VG-06": true,
        "VG-07": true
      }
    }
    ```

---

## 3. Acceptance Criteria

1. Executing `gitmap cluster run-script localhost scripts/deploy-antigravity-ide-fleet.ps1 -DryRun` completes with exit code 0.
2. The Tauri command `execute_gitmap_ide_deploy` successfully dispatches GitMap, capturing and returning structured scorecard results.
3. Remote deployment over SSH correctly invokes PowerShell on the target node with `-ExecutionPolicy Bypass`.
4. Stale or locked files on remote targets do not crash the deployment pipeline; errors are gracefully trapped and logged to `pipeline.db`.

---

## 4. Targeted Verification

- Execute simulated local deployment via GitMap CLI:
  ```powershell
  gitmap cluster run-script localhost scripts/deploy-antigravity-ide-fleet.ps1 -DryRun
  ```
- Verify output contains:
  `FINAL OUTCOME: 7/7 GATES PASSED (100% PRODUCTION-READY)`

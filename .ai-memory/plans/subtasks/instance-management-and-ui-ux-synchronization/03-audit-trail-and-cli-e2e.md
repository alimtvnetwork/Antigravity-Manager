# Subtask Plan: Audit Trail Logging, CLI Protection & E2E Testing

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Plan File:** `.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/03-audit-trail-and-cli-e2e.md`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Spec Reference:** `02-spec/21-app/instance-management-and-ui-ux-synchronization/03-migration-and-verification-spec.md`
- **Status:** `READY`
- **Assigned Authors:** Spec Author 03 / Worker 03
- **Coverage:** Task-04 (Audit Logging), Task-05 (CLI Caller Protection), Task-06 (Test Instance Lifecycle & E2E Suite)

---

## Task-04: Instance Audit Trail Event Logging & Multi-Source Visibility

### Objective
Instrument instance management actions (`LaunchInstance`, `StopInstance`, `RestartInstance`, `SwitchAccount`) to record structured entries in `task_history_db` so that opening the audit trail modal immediately displays rich, chronological event history rather than a blank state.

### Target Files & Precise Locations
1. `src-tauri/src/modules/audit_action.rs` (Lines 1–60):
   - Add enum variants:
     - `LaunchInstance = 6`
     - `StopInstance = 7`
     - `RestartInstance = 8`
     - `KillInstance = 9`
     - `CreateInstance = 10`
     - `DeleteInstance = 11`
   - Implement `code()`, `pascal()`, `title()`, `from_code()`, and `from_legacy()` handlers.
2. `src-tauri/src/modules/instance/launch.rs` (Lines 450–520):
   - Inside `launch_instance_inner_with_extra_workspaces`, invoke `task_history_db::record_task(AuditAction::LaunchInstance, ...)` after successful process spawn and upon error.
3. `src-tauri/src/modules/instance/create.rs` (Lines 325–334):
   - Inside `stop_instance`, record `AuditAction::StopInstance` upon successful verified termination and on failure.
4. `src-tauri/src/modules/instance/lifecycle.rs` (Lines 75–100):
   - Inside `restart_instance`, record `AuditAction::RestartInstance` with details of refreshed credentials and bound account.
5. `src-tauri/src/modules/task_history_db/switch_history.rs` (Lines 40–66, 350–380):
   - Ensure `get_instance_audit_trail` and `get_instance_switch_history` query normalizes aliases (`""`, `"default"`, `"antigravity"`, `"classic"`).
6. `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx` (Lines 29–78, 120–128):
   - Support badge rendering for `LaunchInstance`, `StopInstance`, `RestartInstance`.
   - Preserve automatic fallback to `'operations'` tab when `recent_switches.length === 0` but `trailRes.length > 0`.

### Exact Modification Steps
1. **Audit Enum Update (`audit_action.rs`):**
   - Extend `AuditAction` enum with variants `6..=11`.
   - Update `match` expressions in `pascal()`, `title()`, and `from_code()`.
   - In `from_legacy()`, match `"launchinstance" | "startinstance" | "launch" | "start" | "6"`, `"stopinstance" | "killinstance" | "stop" | "close" | "7"`, `"restartinstance" | "restart" | "8"`.
   - Add unit test in `audit_action.rs::tests` validating bidirectional mapping for new variants.
2. **Instrumentation in `launch.rs`:**
   - In `launch_instance_inner_with_extra_workspaces`, after `is_instance_process_running_smart` confirms process alive:
     ```rust
     crate::modules::task_history_db::record_task(
         crate::modules::audit_action::AuditAction::LaunchInstance,
         &format!("Launch instance {}", instance_id),
         crate::modules::task_history_db::AuditTaskStatus::Succeeded,
         &format!("Launched executable {:?}, data_dir: {}", exe_path, data_dir),
         Some(instance_id),
     );
     ```
3. **Instrumentation in `create.rs`:**
   - In `stop_instance`:
     ```rust
     let res = close_instance_verified(&resolved, std::time::Duration::from_millis(5000));
     match res {
         Ok(true) => {
             crate::modules::task_history_db::record_task(
                 crate::modules::audit_action::AuditAction::StopInstance,
                 &format!("Stop instance {}", resolved),
                 crate::modules::task_history_db::AuditTaskStatus::Succeeded,
                 "Process tree terminated and verified dead",
                 Some(&resolved),
             );
             Ok(())
         }
         Ok(false) => {
             crate::modules::task_history_db::record_task(
                 crate::modules::audit_action::AuditAction::StopInstance,
                 &format!("Stop instance {}", resolved),
                 crate::modules::task_history_db::AuditTaskStatus::Failed,
                 "Process stragglers survived termination ladder",
                 Some(&resolved),
             );
             Err(format!("Instance '{resolved}' is still running after stop"))
         }
         Err(e) => {
             crate::modules::task_history_db::record_task(
                 crate::modules::audit_action::AuditAction::StopInstance,
                 &format!("Stop instance {}", resolved),
                 crate::modules::task_history_db::AuditTaskStatus::Failed,
                 &format!("Termination error: {e}"),
                 Some(&resolved),
             );
             Err(e)
         }
     }
     ```
4. **Instrumentation in `lifecycle.rs`:**
   - In `restart_instance`, record `AuditAction::RestartInstance` upon confirmation.

### Verification Commands & Acceptance Criteria
- Run unit tests: `cd src-tauri && cargo test modules::audit_action`
- Verify audit recording via CLI:
  1. `agm instances stop <test_id>`
  2. Inspect SQLite table: query `SELECT * FROM hot_tasks_cache WHERE action_code = 7`
- Acceptance: Audit modal displays `Launch Instance` and `Stop Instance` badges with green/slate pills without requiring an explicit account rotation.

---

## Task-05: Active IDE Instance Detection (`VSCODE_PID` & Ancestors) & CLI Defense-in-Depth

### Objective
Prevent destructive CLI commands from killing or altering the calling IDE or background agent terminal by implementing multi-tiered caller detection and refusing execution when the target matches the detected caller.

### Target Files & Precise Locations
1. `src-tauri/src/modules/instance/close.rs` (Lines 600–744):
   - Enhance `detect_caller_instance_id()`:
     - Check `ANTIGRAVITY_INSTANCE_ID` and `AGM_INSTANCE_ID`.
     - Check `VSCODE_PID` environment variable and match against registered instance PIDs.
     - Scan 24-level ancestor process tree matching `antigravity.exe` or `code.exe` and parsing `--user-data-dir`.
   - Update `refuse_if_caller_instance(target_id: &str)`:
     - Resolve `target_id` via `resolve_instance_id`.
     - Reject execution if caller ID equals target ID.
2. `src-tauri/src/bin/agm/instance_cmds.rs` (Lines 290–350, 375–385, 435–450):
   - Invoke `guard_caller_instance(&target_instance)` in `cmd_instances_stop`, `cmd_instances_restart`, `cmd_instances_switch`, `cmd_instances_delete`.

### Exact Modification Steps
1. **VSCODE_PID Inspection in `close.rs`:**
   - Insert before process ancestor loop:
     ```rust
     if let Ok(vpid_str) = std::env::var("VSCODE_PID") {
         if let Ok(vpid) = vpid_str.trim().parse::<u32>() {
             if let Some(ref reg) = registry {
                 for inst in &reg.instances {
                     if inst.pid == Some(vpid) {
                         return Some(inst.id.clone());
                     }
                 }
             }
         }
     }
     ```
2. **Ancestor Scanning Refinement:**
   - In ancestor loop, extract `--user-data-dir` or `-user-data-dir` arguments cleanly using helper function `extract_user_data_dir(&cmd)`.
   - Normalize directory paths (`path_matches_dir`) considering Windows short paths and slash direction.
3. **CLI Caller Guard in `instance_cmds.rs`:**
   - Ensure `guard_caller_instance` exits with code 2:
     ```rust
     fn guard_caller_instance(target_instance: &str) {
         if let Err(err) = instance::refuse_if_caller_instance(target_instance) {
             eprintln!("[ERROR] {err}");
             std::process::exit(2);
         }
     }
     ```

### Verification Commands & Acceptance Criteria
- Run probe command: `agm instances current` (outputs detected ID or `none`).
- Run override probe: `$env:ANTIGRAVITY_INSTANCE_ID="caller-probe"; agm instances current` (must output `caller-probe`).
- Run self-stop refusal test: `$env:ANTIGRAVITY_INSTANCE_ID="test-self"; agm instances stop test-self`
  - Must fail with Exit Code 2 and error: `[ERROR] Refusing to stop, switch, or delete instance 'test-self' because this command is running inside its IDE process`.

---

## Task-06: Test Instance CLI Lifecycle & Fast-Failing PowerShell E2E Suite

### Objective
Provide fast, transient test instance commands (`agm instances create-test`, `agm instances rm-test`) and implement a strictly bounded (<30s) end-to-end verification script (`scripts/test-instance-lifecycle-e2e.ps1`).

### Target Files & Precise Locations
1. `src-tauri/src/bin/agm/instance_cmds.rs` (Lines 480–660):
   - `cmd_instances_create_test`: Allocate isolated `%LOCALAPPDATA%/agm-instances/test-<id>`, register in `registry.json`, output JSON envelope.
   - `cmd_instances_rm_test`: Cleanly kill processes, delete directory, remove from `registry.json`.
2. `scripts/test-instance-lifecycle-e2e.ps1` (New File):
   - PowerShell script automating the end-to-end test protocol.

### Exact Modification Steps
1. **CLI Commands Review (`instance_cmds.rs`):**
   - Ensure `cmd_instances_create_test` and `cmd_instances_rm_test` support both human-readable and `--json` flags.
   - Ensure `rm-test` cleans all transient lockfiles before unregistering.
2. **Author `scripts/test-instance-lifecycle-e2e.ps1`:**
   - Configure `$ErrorActionPreference = "Stop"`.
   - Resolve binary via relative path: `src-tauri/target/debug/agm.exe`.
   - Implement Step 1: Detect caller instance via `agm instances current`.
   - Implement Step 2: Test self-preservation guard rejection (assert exit code 2).
   - Implement Step 3: Create transient test instance via `agm instances create-test --name "e2e-sandbox" --json`.
   - Implement Step 4: Verify filesystem directory on disk and entry in `registry.json`.
   - Implement Step 5: Test stopping instance and verify process termination.
   - Implement Step 6: Verify audit event written to `task_history_db`.
   - Implement Step 7: Clean up via `agm instances rm-test` and verify zero leftover files.
   - Enforce bounded timeouts on each step:
     ```powershell
     $job = Start-Job -ScriptBlock { & $args[0] @($args[1..($args.Length-1)]) } -ArgumentList @($agmExe, "instances", "current")
     if (-not (Wait-Job $job -Timeout 15)) {
         Stop-Job $job
         throw "Command timed out after 15 seconds"
     }
     ```

### Verification Commands & Acceptance Criteria
- Run E2E script locally: `powershell -ExecutionPolicy Bypass -File scripts/test-instance-lifecycle-e2e.ps1`
- Script completes with exit code 0 in under 30 seconds.
- Zero leftover directories in `%LOCALAPPDATA%/agm-instances/test-*`.
- Rust preflight passes: `cd src-tauri && cargo clippy --all-targets --all-features` and `cargo fmt -- --check`.

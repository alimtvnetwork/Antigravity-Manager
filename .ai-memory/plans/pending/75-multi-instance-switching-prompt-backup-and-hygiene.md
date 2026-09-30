# Plan 75: Multi-Instance Isolation, Fast-Forward Switching, Prompt Resumption & Developer Hygiene

> **Originating Request:** User prompt instruction synthesis from reported field issues.  
> **Target Subsystems:** `instance.rs`, `auto_switcher.rs`, `repo_db.rs`, `backup_prompts_db.rs`, `scripts/dev-tool-clear.ps1`, `scripts/test-instance-e2e.ps1`.  
> **Release Target:** Minor Version Bump (`npm run bump minor`) with full CI/CD verification.

---

## 📋 Master Prompt Instruction (Executable Specification)

```markdown
/goal Autonomously diagnose, fix, end-to-end verify, and release the Multi-Instance Sandboxing, Profile Switching, Running Prompt Resumption, and Developer Hygiene subsystems in Antigravity-Manager.

## 🎯 Executive Objective & Problem Classification

Multiple critical failures have been identified in the Multi-Instance and Fast-Forward subsystems:
1. **Multi-Instance Account Contamination (Same Email Bug):** Creating or cloning an isolated instance (e.g., Instance B from Instance A) results in both instances sharing the default profile's email/credentials instead of maintaining strictly sandboxed, independent accounts.
2. **Account Switch / Fast-Forward Non-Progression:** Initiating a profile switch or fast-forward on an instance fails to bind and activate the newest rotated account. Even after rotating 2–3 times, the IDE restarts with the previous/stale account still active.
3. **Loss of Running Prompts Across IDE Restarts:** Terminating the IDE during an account switch currently drops active instruction tasks. Running prompts must be backed up before IDE termination, preserved across account swaps, and automatically re-injected/resumed upon IDE launch.
4. **Developer Hygiene Gap for Rust/Cargo:** `scripts/dev-tool-clear.ps1` must be extended to thoroughly purge Cargo/Rust build artifacts, caches, and intermediate targets across both the root and `src-tauri` directory.
5. **Lack of Automated Local Multi-Instance E2E Verification:** Previous fixes were claimed without real multi-instance creation and teardown. You must locally create multiple sandboxed instances, verify account distinctness, test switching, confirm prompt resumption, and cleanly destroy the test instances post-test.
6. **Minor Release & CI/CD Green Gate:** Once verified, perform a **minor** version release (`npm run bump minor`), enforce `@aukgit` changelog attribution, and monitor remote CI/CD pipelines until completely green.

---

## 📐 Strict System Invariants (Non-Negotiable)

- **[Invariant 1 - Instance Credential Sandboxing]:** No secondary instance may ever read from or write to the system OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service). Keyring bypass markers (`antigravity-keyring-unavailable`, `antigravity-ide-keyring-unavailable`, `antigravity-cli-keyring-unavailable`) and SSH emulation environment variables must be actively enforced. Credentials must be isolated to that instance's 3-database storage (`<data_dir>/User/globalStorage/state.vscdb`, `<data_dir>/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`, `<home_dir>/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`).
- **[Invariant 2 - Fast-Forward Account Progression]:** When a profile switch or fast-forward is triggered on an instance, the target instance's `state.vscdb` credentials, `app_storage.json`, and environment bindings must be explicitly overwritten with the selected candidate account credentials before IDE relaunch. Volatile session directories (`Local Storage`, `Session Storage`, `Cache`, `GPUCache`) must be purged to prevent Electron from loading cached tokens.
- **[Invariant 3 - Zero Prompt Loss & Continuous Telemetry]:** Active prompts must be transitioned to `backed_up` in `repo_prompts.db` and committed to `backup-prompts.db` prior to closing the IDE. An atomic `.antigravity_resume_task.json` must be seeded in the workspace root. Upon restart, prompts must be re-injected via the agy bridge, and `.antigravity_goal_prompt.log` heartbeats must continue seamlessly at `Iteration: N+1` without resetting.
- **[Invariant 4 - Zero Test Artifact Leftovers]:** Any temporary test instances created during E2E verification (e.g. `test-inst-alpha`, `test-inst-beta`) must be terminated and completely removed (`agm instances rm <id> --force`) upon test completion.
- **[Invariant 5 - Strict Release & Attribution Discipline]:** For the minor release, follow `agm-release-lifecycle`. Attribution in `CHANGELOG.md`, `changelog_en.md`, `README.md`, and `README_EN.md` must attribute **strictly and exclusively** `@aukgit`. No other usernames or external handles may appear.
- **[Invariant 6 - Green CI/CD Loop]:** Do not declare completion until GitHub Actions CI/CD workflows are completely green. If a CI check fails, perform root cause analysis, fix the issue, and repeat until the pipeline passes.

---

## 🛠️ Step-by-Step Implementation & Verification Plan

### Phase 1: Deep Root Cause Analysis & Code Audit
1. Audit `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/repo_db.rs`, and `src-tauri/src/commands/instance.rs`.
2. Trace the exact code path where an instance is launched and where account switching occurs:
   - Identify why a cloned or newly created instance inherits the default profile's email (check environment variable inheritance, default profile path fallbacks, and keyring bypass initialization).
   - Identify why account rotation does not take effect inside the instance (check if `cleanAndRestartWorkspace` or `switch_instance_account` is writing to the default profile rather than the target instance profile directory, or if Electron cache is retaining the previous session).
3. Trace `backup_running_prompts` and `resend_running_commands_for_instance` in `repo_db.rs` to identify why running tasks fail to resume after an instance switch.

### Phase 2: Surgical Code Refactoring
1. **Fix Multi-Instance Account Isolation:**
   - Ensure that whenever a new instance is created or cloned, its credentials in all 3 `state.vscdb` databases and `app_storage.json` are isolated and populated strictly with the designated account (or left unauthenticated), never copying or leaking default profile tokens.
   - Verify that all three keyring bypass markers (`antigravity-keyring-unavailable`, `antigravity-ide-keyring-unavailable`, `antigravity-cli-keyring-unavailable`) are physically present in `<instance_dir>/data` prior to process spawning.
2. **Fix Account Switching & Fast-Forward Progression:**
   - In `auto_switcher.rs` and `instance.rs`, ensure that switching an instance explicitly locates that instance's sandboxed `data` and `home` directories.
   - Write the new account tokens and user details into the instance's 3-database locations.
   - Purge volatile session caches (`Local Storage`, `Session Storage`, `Network`, `blob_storage`, `Cache`) in the instance directory to prevent Electron memory persistence.
3. **Harden Running Prompt Backup & Resumption:**
   - Guarantee that before IDE termination, `backup_running_prompts(instance_id)` transitions running tasks to `backed_up` and writes atomic `.antigravity_resume_task.json` into the bound workspace root.
   - On IDE relaunch, automatically re-inject the prompt via `resend_running_commands_for_instance` / agy bridge and ensure the prompt heartbeat runner (`scripts/prompt_heartbeat_runner.py`) resumes at `Iteration: N+1`.
4. **Enhance Developer Hygiene (`scripts/dev-tool-clear.ps1`):**
   - In `scripts/dev-tool-clear.ps1`, add comprehensive Cargo and Rust clearing:
     - Purge `src-tauri/target/debug/incremental`, `src-tauri/target/release/incremental`, and intermediate build cache directories.
     - Support full Cargo cleaning (`cargo clean` in `src-tauri` when `--cargo-only` or `--force` is passed).
     - Add clearing of test instance artifacts, lockfiles, and temporary prompt logs.

### Phase 3: Comprehensive Local Multi-Instance E2E Verification
Author or update an automated end-to-end test script (e.g. in `scripts/test-instance-e2e.ps1` or a dedicated test harness) that runs locally on Windows and executes the following verification flow:
1. **Instance Creation & Distinct Account Verification:**
   - Create Instance 1 (`test-inst-1`) and bind Account A.
   - Create Instance 2 (`test-inst-2`) and bind Account B.
   - Verify by inspecting the sandboxed `state.vscdb` files that Instance 1 has Account A's email and Instance 2 has Account B's email, with **zero** cross-contamination.
2. **Account Switching & Fast-Forward Verification:**
   - Trigger account rotation / switch on Instance 1 to Account C.
   - Verify that Instance 1's `state.vscdb` now reflects Account C.
   - Rotate again to Account D; verify that Instance 1 updates to Account D.
3. **Running Prompt Backup & Resumption Verification:**
   - Seed a mock active prompt for Instance 1.
   - Initiate instance switch/restart.
   - Assert that the prompt was backed up to `repo_prompts.db`, `.antigravity_resume_task.json` was generated in the workspace, and the prompt was queued for re-injection post-switch.
4. **Mandatory Teardown & Cleanup:**
   - Terminate all test instance processes.
   - Force-remove `test-inst-1` and `test-inst-2` (`agm instances rm <id> --force`).
   - Run `scripts/dev-tool-clear.ps1` and verify that all test instances, lockfiles, and caches are 100% clean.

### Phase 4: Pre-flight, Minor Version Release & CI/CD Verification
1. Run local pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. Perform minor release bump:
   - Run `npm run bump minor` to synchronize versions across all manifests.
   - Synchronize changelogs (`CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md`) with release notes. Ensure attribution is **strictly and exclusively** `@aukgit` (`(Thanks to @aukgit)`).
3. Commit, tag, and push:
   - Push to `main` branch with the appropriate tag `vX.Y.Z`.
4. CI/CD Monitoring Loop:
   - Monitor GitHub Actions workflow runs until all jobs (build, package, test) succeed and show green.
   - If any CI step fails, diagnose, fix, and re-verify until 100% successful.

---

## 📋 Acceptance Criteria Checklist

- [ ] Multiple instances created locally have distinct, isolated emails; no fallback to the default profile account.
- [ ] Switching or fast-forwarding an instance successfully updates the instance's account to the newest selected candidate across repeated rotations.
- [ ] Running prompts are snapshotted prior to IDE termination and re-injected automatically upon restart without dropping state.
- [ ] `scripts/dev-tool-clear.ps1` contains full Cargo/Rust cache clearing capabilities.
- [ ] Local E2E multi-instance test suite executes end-to-end and cleans up all created instances upon completion.
- [ ] Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`) pass with zero warnings or errors.
- [ ] Version is bumped as a minor release (`npm run bump minor`), with README and changelog files synchronized under strict `@aukgit` attribution.
- [ ] CI/CD pipeline builds pass completely green on GitHub Actions.
/goal
```

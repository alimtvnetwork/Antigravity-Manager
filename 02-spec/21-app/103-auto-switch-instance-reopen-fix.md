# Application Specification: 103 - Auto-Switch Instance Reopen & Cross-Instance Rotation Fix

**Version:** 1.0.0  
**Status:** Completed & Verified  
**Author:** Antigravity Engineering  
**Scope:** `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/integration.rs`, `src-tauri/src/models/config.rs`, `src/pages/Instances.tsx`, `src/stores/useInstanceStore.ts`  
**Reference Screenshot:** `assets/screenshots/103-auto-switch-instance-reopen-fix-01.png`

---

## 1. Executive Summary & Problem Definition

When an active instance (such as the default workspace `#1 Default`) exhausts its quota (0% credits or drops below the critical threshold `<= 15%`), the Auto Profile Switcher detects the quota deficit and attempts to rotate. However, the system fails to reopen to the new candidate instance (e.g. `#2 Default Copy`), leaving the depleted instance in `Status: Idle` with no running IDE window.

### User Bug Symptoms
1. Instance `#1 Default` reaches 0% credits for Gemini 3.1 Pro (High).
2. The switcher banner displays:
   `"Auto Profile Switcher Active ... Last switch: Critical quota alert on instance 'default': model 'gemini-flash' dropped to 8.0% (<= 15.0%). Fast-forwarding to highest credit candidate..."`
3. The instance selector at the top reflects `#2 Default Copy`, but neither the depleted instance was cleanly rotated nor was the target instance window opened/reopened.
4. The user is left with a dead/idle workspace and no running IDE process.

---

## 2. Root Cause Analysis (RCA)

### RC-1: Candidate Instance ID Overwritten with `current_instance_id`
- **Location:** `src-tauri/src/modules/auto_switcher.rs` (Lines 879–886, 923, 950)
- **Defect:** In `select_candidate_profiles`, when iterating through accounts registered under other instances in `registry.instances`, line 880 pushes:
  ```rust
  candidates.push(ProfileCandidate {
      instance_id: current_instance_id.to_string(), // BUG: Overwrites inst.id with current ('default')!
      account_id: acc.id.clone(),
      email: acc.email.clone(),
      ...
  });
  ```
- **Consequence:** The actual owner instance ID (e.g. `instance_2` / `default-copy`) is destroyed and replaced with `'default'`. The rotation logic believes the account belongs to `'default'`, bypassing cross-instance activation and window launching.

### RC-2: Missing Cross-Instance Process Launch in Auto-Switcher
- **Location:** `src-tauri/src/modules/auto_switcher.rs` (Lines 1285–1305)
- **Defect:** `execute_profile_rotation_with_context` only executes credential injection via `service.switch_account` or `instance::switch_account_to_instance`. It contains ZERO calls to `instance::launch_instance`, `instance::close_instance`, or `instance::set_active_instance_id`.
- **Consequence:** Even if a target instance is chosen, its window is never launched, and the depleted instance process is not closed cleanly.

### RC-3: Premature Short-Circuit in `switch_account_to_instance`
- **Location:** `src-tauri/src/modules/instance.rs` (Lines 3037–3055)
- **Defect:** When `switch_account_to_instance` is called for `is_default_inst`, lines 3040–3054 delegate to `service.switch_account` and immediately return `Ok(())` without executing the process restart, window reopening, and prompt restoration pipeline implemented at lines 3330–3345.
- **Consequence:** The default instance's Antigravity IDE is killed, but never relaunched via `launch_instance_without_prompt_reinject("default")`, leaving `Status: Idle`.

### RC-4: Lockfile Collisions & Process Group Spawning on Windows
- **Location:** `src-tauri/src/modules/integration.rs` (Lines 356–388) & `src-tauri/src/modules/process.rs`
- **Defect:** `DesktopIntegration::on_account_switch` calls `close_instance("default")` which runs `taskkill /F /T`. Immediately thereafter, without clearing stale `lockfile` / `code.lock` in AppData or allowing Windows OS handles to close, it calls `start_antigravity_with_fallback_path`. On Windows, the process is spawned without detached process flags, and Electron immediately aborts due to locked cache files.

---

## 3. Non-Negotiable Invariants

### Invariant 1: Preserve Candidate Instance Ownership
In `auto_switcher::select_candidate_profiles`, candidate profiles MUST preserve their owning `instance_id` (`inst.id.clone()`).

### Invariant 2: Unified 5-Step Cross-Instance Lifecycle
When auto-switch selects a candidate belonging to a different instance (`target.instance_id != current_instance_id`):
1. **Step 1 (Backup):** Call `requeue_running_conversations_for_instance(current_instance_id)` to save in-flight prompts.
2. **Step 2 (Close):** Call `instance::close_instance(current_instance_id)` to gracefully stop the depleted instance.
3. **Step 3 (Activate):** Call `instance::set_active_instance_id(&target.instance_id)` and update registry active pointer.
4. **Step 4 (Launch/Reopen):** Call `instance::launch_instance(&target.instance_id)` with bound workspace directories, clearing stale lockfiles.
5. **Step 5 (Resume):** Wait for the instance prompt channel settle delay and auto-dispatch enqueued prompts.

### Invariant 3: Reliable Default Instance Relaunch
In `instance::switch_account_to_instance`, if `is_default_inst` is true and `auto_reopen` is enabled:
- If Antigravity was previously running, ensure `launch_instance_without_prompt_reinject("default")` is invoked.
- Clear stale `lockfile`, `code.lock`, and `DevToolsActivePort` before launching.
- Verify child PID is recorded in `instance_pids`.

### Invariant 4: `auto_reopen_on_switch` Configuration Flag
Add `pub auto_reopen_on_switch: bool` (default `true`) to `AutoProfileSwitcherConfig` in `src-tauri/src/models/config.rs` and expose in GUI settings.

---

## 4. Verification & Release Gate
- `cargo fmt -- --check` must pass with exit code 0.
- `npm run build` must pass with exit code 0.
- Version bump to next patch version via `npm run bump patch` and synchronized changelogs attributed strictly to `@aukgit`.

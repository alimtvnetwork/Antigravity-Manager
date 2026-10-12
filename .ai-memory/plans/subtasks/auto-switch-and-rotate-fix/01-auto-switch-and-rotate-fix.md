# Subtask Implementation Plan: Post-8873a197 Instance Lifecycle, Process Cache, Default Rotation & Frontend Smart Fallback Fixes

> **Subtask ID:** `01-auto-switch-and-rotate-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Parent Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`  
> **Remediation Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/02-rca-and-remediation-spec.md`  
> **Strictly Owned Target Files (`Worker 01`):**
> 1. `src-tauri/src/modules/instance/launch.rs`
> 2. `src-tauri/src/modules/instance/close.rs`
> 3. `src-tauri/src/modules/instance/process_cache.rs`
> 4. `src-tauri/src/modules/instance/switch.rs`
> 5. `src-tauri/src/modules/auto_switcher/rotation.rs`
> 6. `src/services/instanceService/scoring.ts`
> 7. `src/stores/useInstanceStore/smart.ts`
> 8. `src/stores/useAccountStore.ts`
> **Status:** `[READY_FOR_WORKER]`  
> **Author:** @aukgit  
> **Assigned Role:** Worker 01 (Backend Instance Lifecycle & Frontend Smart Rotation Specialist)

---

## 1. Context & Comparative Root Cause Analysis (`v4.162.0` vs `HEAD` Beyond Commit `8873a197`)

While commit `8873a197` resolved cross-instance child PID descent in `pid_scan.rs:302` and backend Step 3 fallback scoring in `selection.rs:294`, a comprehensive line-by-line architectural audit comparing baseline `v4.162.0` (`src-tauri/src/modules/instance.rs`) against `HEAD` uncovered **4 additional root causes (Defects A1–A4)** that prevent clean instance termination, lockfile purging, default instance rotation, and frontend UI Fast-Forward / smart rotation:

### 1.1 DEFECT A1 (`src-tauri/src/modules/instance/launch.rs:53-76`): Conditional Gate Bypassing Pre-Launch `close_instance` & Lockfile Purge
- **`v4.162.0` Baseline (`src-tauri/src/modules/instance.rs:3199-3225`)**: `launch_instance` unconditionally called `close_instance(instance_id)` and unconditionally purged stale lockfiles (`lockfile`, `code.lock`, `Singleton*`, `*.lock`) before injecting credentials into `state.vscdb` and spawning the IDE.
- **`HEAD` Defect (`src-tauri/src/modules/instance/launch.rs:53-76`)**: `close_instance` and `clear_stale_instance_lockfiles` are gated behind `if !is_already_running`, and `close_instance` is further gated behind `if !has_other_alive`:
  - Whenever any sibling instance is running (`has_other_alive == true`), `launch_instance` skips `close_instance(instance_id)`—even though `close_instance` already protects sibling PIDs internally via `should_spare_pid`.
  - Whenever `is_already_running == true` due to a surviving helper process or stale cache during `restart_instance` / `switch_account_to_instance`, `launch_instance` skips `clear_stale_instance_lockfiles(&target_data_path)` and attempts to inject into a locked `state.vscdb` and spawn a duplicate colliding process.

### 1.2 DEFECT A2 (`src-tauri/src/modules/instance/close.rs:71-225` & `src-tauri/src/modules/instance/process_cache.rs:30-56`): Stale 4000ms Process Cache on Close & Orphan Child PID Promotion
- **`HEAD` Defect (`close.rs:71-225`)**: `find_pids_for_data_dir` reads `get_cached_antigravity_processes()` (4000ms cache TTL) without `close_instance` or `close_instance_verified` calling `force_refresh_process_cache()` first. Consequently, `close_instance` and the verification loop in `close_instance_verified` inspect a stale process snapshot for up to 4 seconds.
- **`HEAD` Defect (`process_cache.rs:30-56`)**: `get_cached_instance_process_state` promotes any surviving `child_pids` entry (e.g., `crashpad_handler` or utility process) to `primary_pid` via `.or_else(|| matched_entry.and_then(|e| e.child_pids.first().copied()))` when the main window exits (`root_pids` is empty), falsely reporting `running = true`.

### 1.3 DEFECT A3 (`src-tauri/src/modules/instance/switch.rs:321-340` & `src-tauri/src/modules/auto_switcher/rotation.rs:298-309`): Swallowed Relaunch Errors & Unverified Default Instance Switch
- **`HEAD` Defect (`switch.rs:321-340`)**: `switch_account_to_instance` swallows relaunch failures (`if let Err(launch_err)` logs a warning and returns `Ok(())` with `switch_ok: true` instead of propagating the error as in `v4.162.0`). This violates the No Silent Errors invariant and reports fake success when relaunch fails.
- **`HEAD` Defect (`rotation.rs:298-309`)**: `default` instance switches in `rotation.rs:298-309` bypass `switch_account_to_instance` (`close_instance_verified` + `clear_stale_instance_lockfiles` + `inject_account_credentials` + `record_instance_pid`), routing instead through `DesktopIntegration::on_account_switch`, which performs unverified close and partial lockfile cleanup.

### 1.4 DEFECT A4 (`src/services/instanceService/scoring.ts`, `src/stores/useInstanceStore/smart.ts`, `src/stores/useAccountStore.ts`): Frontend Hard 100% Quota Gate, Stale `is_running` Duplicate Launch & Missing Store Refresh
- **`HEAD` Defect (`scoring.ts` & `smart.ts`)**: Frontend `scoring.ts` and `smart.ts` enforce a hard `fourHourQuotaPercent >= 100` gate with zero sub-100% fallback (unlike backend `selection.rs` Step 3), causing UI Fast-Forward (`InstanceSelector.tsx`, `useFastForwardShortcut.ts`) and `smartRotateProfileAccount` to throw `'No accounts verified with 100% 4-hour quota in pool'` whenever all accounts have `< 100%` quota.
- **`HEAD` Defect (`smart.ts:262-267`)**: `smart.ts` checks `get().instances.find(...)?.is_running` **before** calling `fetchInstances(true)`, reading stale `false` after `switchAccount` already relaunched the instance on the backend and triggering a colliding duplicate `launchInstance` call.
- **`HEAD` Defect (`useAccountStore.ts:107-117`)**: `useAccountStore.switchAccount` fails to refresh `fetchInstances(true)` and `fetchAccounts()` after `accountService.switchAccount` completes.

---

## 2. File Boundaries & Safety Rules (`Worker 01`)

- **Strictly Owned Files**: `Worker 01` may ONLY modify the 8 files listed in the header above.
- **No Silent Errors**: Route every fallible operation in Rust through `AppError` or `crate::error::record_ignored(..., "...")` with a `// Justification:` comment. Route every best-effort catch in TypeScript through `useErrorStore.getState().trackWarning(e, { source: ..., triggerAction: ... })`.
- **Zero Git & Search Commands**: NEVER run raw `git` commands (`git status`, `git diff`, `git add`, `git commit`, etc.) or raw search tools (`rg`, `grep`, `Select-String`, `findstr`).

---

## 3. Step-by-Step Implementation Checklist for `Worker 01`

### Step 1: Unconditional Pre-Launch Close & Lockfile Purge (`src-tauri/src/modules/instance/launch.rs`)
- [ ] View `src-tauri/src/modules/instance/launch.rs` around lines 45–85.
- [ ] Remove the `if !has_other_alive` gate around `close_instance(instance_id)` and ensure that before credential injection (`inject_account_credentials`), `close_instance(instance_id)` and `clear_stale_instance_lockfiles(&target_data_path)` are executed unconditionally whenever launching or relaunching an instance (matching `v4.162.0` `instance.rs:3199-3225`), unless the caller explicitly requested a pure focus-only operation on a verified live root window without relaunch:
  - Specifically, in `launch_instance(instance_id: &str, force_new_window: bool)`:
    - When `force_new_window` is `true` (used by `switch_account_to_instance` and `restart_instance`) or when `!is_already_running`, unconditionally call `close_instance(instance_id)` (recording any ignored cleanup result via `crate::error::record_ignored` with a `// Justification:` comment, as `close_instance` already protects sibling instance PIDs via `should_spare_pid`) and unconditionally call `clear_stale_instance_lockfiles(&target_data_path)` prior to `inject_account_credentials`.
    - Even when `is_already_running` is false and `has_other_alive` is true, `close_instance(instance_id)` and `clear_stale_instance_lockfiles(&target_data_path)` MUST run so orphan processes and stale `Singleton*`/`*.lock` files are purged without being blocked by running sibling instances.

### Step 2: Refresh Process Cache in Close Flows (`src-tauri/src/modules/instance/close.rs`)
- [ ] View `src-tauri/src/modules/instance/close.rs` around lines 71–225 (`close_instance_verified` and `close_instance`).
- [ ] At the entry of `close_instance(instance_id: &str)`, call `crate::modules::instance::process_cache::force_refresh_process_cache();` before scanning PIDs via `find_pids_for_data_dir`.
- [ ] Inside `close_instance_verified(instance_id: &str, timeout_ms: u64)`:
  - Call `crate::modules::instance::process_cache::force_refresh_process_cache();` before each `find_pids_for_data_dir` check in the polling verification loop so that process termination detection queries live OS process state rather than a 4000ms cached snapshot.
  - Call `crate::modules::instance::process_cache::force_refresh_process_cache();` once termination is verified (or after the final `kill_pids_for_data_dir` fallback) so subsequent `launch_instance` or status queries observe an immediately clean cache.

### Step 3: Prevent Orphan `child_pids` Promotion to `primary_pid` (`src-tauri/src/modules/instance/process_cache.rs`)
- [ ] View `src-tauri/src/modules/instance/process_cache.rs` around lines 30–56 (`get_cached_instance_process_state`).
- [ ] Inspect how `primary_pid` and `running` are computed from `matched_entry`.
- [ ] Remove the `.or_else(|| matched_entry.and_then(|e| e.child_pids.first().copied()))` fallback that promotes an orphaned child process (such as `crashpad_handler` or utility worker) to `primary_pid` when `root_pids` is empty:
  ```rust
  let primary_pid = matched_entry.and_then(|e| {
      if let Some(h_pid) = hint_pid {
          if e.root_pids.contains(&h_pid) {
              return Some(h_pid);
          }
      }
      e.root_pids.first().copied()
  });
  let running = primary_pid.is_some();
  ```
- [ ] Ensure `running` is `true` only when a genuine root IDE process exists in `root_pids`.

### Step 4: Propagate Relaunch Errors in `switch_account_to_instance` (`src-tauri/src/modules/instance/switch.rs`)
- [ ] View `src-tauri/src/modules/instance/switch.rs` around lines 315–350.
- [ ] Replace the warning-only `if let Err(launch_err) = launch_instance(target_id, true)` block with proper error propagation (matching `v4.162.0` and the No Silent Errors invariant):
  ```rust
  if was_running {
      tracing::info!(
          "[Switch] Step 4/4: Instance '{}' was running before switch; launching fresh...",
          target_id
      );
      let launched = launch_instance(target_id, true).map_err(|launch_err| {
          let msg = format!(
              "Account credentials injected into '{}', but relaunch failed: {}",
              target_id, launch_err
          );
          tracing::error!("[Switch] {}", msg);
          msg
      })?;
      final_pid = launched.pid;
  }
  ```
- [ ] Verify that `final_pid` captures the newly launched instance PID when `was_running` is true, and that any relaunch failure returns `Err(...)` instead of fake `Ok(InstanceSwitchOutcome { switch_ok: true, ... })`.

### Step 5: Unify `default` Instance Rotation in Auto-Switcher (`src-tauri/src/modules/auto_switcher/rotation.rs`)
- [ ] View `src-tauri/src/modules/auto_switcher/rotation.rs` around lines 290–320.
- [ ] In `execute_profile_rotation_with_context`, inspect the `if target_instance == "default"` branch (lines 298–309) which currently bypasses `switch_account_to_instance` and calls `DesktopIntegration::on_account_switch(&best_account)`.
- [ ] Route `"default"` instance rotation through `crate::modules::instance::switch_account_to_instance("default", &best_account.id)` so that `"default"` receives the exact same `close_instance_verified` + `clear_stale_instance_lockfiles` + `inject_account_credentials` + `record_instance_pid` pipeline as named instances, and sync desktop integration metadata without performing a second unverified close/launch race.

### Step 6: Add Sub-100% Proportional Fallback Scoring in Frontend (`src/services/instanceService/scoring.ts`)
- [ ] View `src/services/instanceService/scoring.ts`.
- [ ] Keep `scoreAccountForSelection(account: Account, modelName?: string): number` for primary 100% 4-hour quota evaluation, and add a new exported fallback scorer `scoreAccountForSelectionFallback(account: Account, modelName?: string): number` (matching backend `score_candidate_account_fallback` in `src-tauri/src/modules/auto_switcher/scoring.rs`):
  - Check `if (account.disabled || account.Quota?.is_forbidden) return 0;`
  - Check weekly quota gate: `const weeklyQuota = getWeeklyRemainingPercent(account); if (weeklyQuota < 8) return 0;`
  - Check 4-hour quota gate: `const fourHour = extractFourHourSnapshot(account, modelName);`
  - If `fourHour.percentage <= 0 && !fourHour.isResetFinished`, return `0`.
  - Compute the base weekly score (`(tierMultiplier * effectiveWeeklyPct * remainingHeadroomHours) / 100.0`) and scale by `(effectiveFourHourPct / 100.0)` where `effectiveFourHourPct = fourHour.isResetFinished ? 100 : fourHour.percentage`.
- [ ] Update `selectBestAccountForInstance` (or any helper in `scoring.ts`) so that if no candidate scores `> 0` with `scoreAccountForSelection`, it falls back to ranking candidates with `scoreAccountForSelectionFallback`.

### Step 7: Implement Two-Tier Fallback & Fix Stale `is_running` Duplicate Launch in `src/stores/useInstanceStore/smart.ts`
- [ ] View `src/stores/useInstanceStore/smart.ts` around lines 140–275.
- [ ] Import `scoreAccountForSelectionFallback` from `../../services/instanceService/scoring`.
- [ ] In `smartRotateProfileAccount` (and any shared candidate selector in `smart.ts`):
  1. **Candidate Pool Fallback**: When building `scoredCandidates`, first score eligible accounts with `scoreAccountForSelection`. If no accounts have a primary score `> 0` (i.e., all accounts in the pool have `< 100%` 4-hour quota), score eligible accounts with `scoreAccountForSelectionFallback` instead of immediately failing.
  2. **Live Verification Fallback**: During the live quota verification loop (`fetchAccountQuota`), if a candidate refreshes at `freshFourHour.percentage >= 100 || freshFourHour.isResetFinished`, select it immediately. Otherwise, if `freshFourHour.percentage > 0 && freshWeekly >= 8`, track the best fallback candidate with the highest `scoreAccountForSelectionFallback(freshAcc)`. If the loop finishes without a `>= 100%` candidate, select the best verified fallback candidate rather than throwing `'No accounts verified with 100% 4-hour quota in pool'`. Only throw if zero candidates have `> 0%` 4-hour quota and `>= 8%` weekly quota.
  3. **Fix Stale `is_running` Duplicate Launch Race (`smart.ts:260-272`)**:
     - After `await accountStore.switchAccount(selectedAccount.email, { instanceId });`, call `await get().fetchInstances(true);` and `await accountStore.fetchAccounts();` **BEFORE** reading `const instAfter = get().instances.find((i) => i.id === instanceId);`.
     - Only call `await get().launchInstance(instanceId)` if `!instAfter?.is_running` after the fresh `fetchInstances(true)` completes.

### Step 8: Refresh Instances & Accounts After `switchAccount` (`src/stores/useAccountStore.ts`)
- [ ] View `src/stores/useAccountStore.ts` around lines 107–125 (`switchAccount`).
- [ ] After `await accountService.switchAccount(email, options)` succeeds and `currentAccount` is updated:
  - Refresh accounts via `await get().fetchAccounts()` (wrapping any non-fatal refresh warning with `useErrorStore.getState().trackWarning`).
  - Refresh instances via `await useInstanceStore.getState().fetchInstances(true)` (wrapping any non-fatal refresh warning with `useErrorStore.getState().trackWarning`) so that instance `is_running`, `pid`, and `last_account` states in Zustand are always immediately synchronized after any account switch.

---

## 4. Acceptance Criteria (`Worker 01`)

- [ ] **Defect A1 Resolved**: `launch_instance` in `src-tauri/src/modules/instance/launch.rs` unconditionally executes `close_instance(instance_id)` and `clear_stale_instance_lockfiles(&target_data_path)` before injecting credentials when launching/relaunching, even when sibling instances are running (`has_other_alive == true`).
- [ ] **Defect A2 Resolved**: `close_instance` and `close_instance_verified` in `src-tauri/src/modules/instance/close.rs` call `force_refresh_process_cache()`, and `get_cached_instance_process_state` in `src-tauri/src/modules/instance/process_cache.rs` never promotes `child_pids` to `primary_pid` when `root_pids` is empty.
- [ ] **Defect A3 Resolved**: `switch_account_to_instance` in `src-tauri/src/modules/instance/switch.rs` propagates `launch_instance` failures as `Err(...)`, and `execute_profile_rotation_with_context` in `src-tauri/src/modules/auto_switcher/rotation.rs` routes `"default"` instance rotation through `switch_account_to_instance("default", ...)`.
- [ ] **Defect A4 Resolved**: Frontend `scoring.ts` and `smart.ts` fall back to proportional sub-100% 4-hour quota scoring when no 100% account exists; `smart.ts` calls `await get().fetchInstances(true)` before checking `instAfter?.is_running`; and `useAccountStore.ts` refreshes both accounts and instances after `switchAccount`.

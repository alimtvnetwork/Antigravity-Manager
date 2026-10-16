---
plan: auto-switch-and-ui-functionality-issues
subtask: "01"
title: Quota Synchronization, Auto-Switch Hardening, Fast-Forward Unblocking, and Backup Prompts Retention Cap
domain: backend-rust-and-frontend-react
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/01-architecture-and-quota-autoswitch-spec.md
  master_plan: ../../auto-switch-and-ui-functionality-issues.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/quota/fetch_more.rs
  - src-tauri/src/modules/quota/fetch_cached.rs
  - src-tauri/src/modules/auto_switcher/daemon.rs
  - src-tauri/src/modules/auto_switcher/status.rs
  - src-tauri/src/modules/auto_switcher/manual.rs
  - src-tauri/src/modules/auto_switcher/selection.rs
  - src-tauri/src/modules/instance/switch.rs
  - src-tauri/src/modules/backup_prompts_db/backup.rs
  - src-tauri/src/models/config.rs
  - src/pages/instances/useInstancePage.ts
  - src/components/settings/AdvancedMaintenanceSection.tsx
status: pending
---

# Subtask 01: Quota Synchronization, Auto-Switch Hardening, Fast-Forward Unblocking, and Backup Prompts Retention Cap

## 1. Context & Motivation

In Antigravity-Manager v4.207.0:
1. When Google's `retrieveUserQuotaSummary` endpoint returns HTTP 403 Forbidden because a provided `project_id` has insufficient permissions, `fetch_more.rs` drops the response and returns `None`. Consequently, `fetch_cached.rs` never fuses genuine 5-hour quota buckets into `ModelQuota`, leaving the UI displaying a false `Gemini 2.5 Pro (4H): 100%` when the IDE is at 0%.
2. Because the quota is falsely reported as 100%, the Auto-Switcher daemon never triggers rotation (< 15%). Furthermore, when the daemon updates quotas in the background, it does not emit `accounts://refreshed`, and `useInstancePage.ts` does not poll `fetchAccounts()`.
3. When manual Fast-Forward is clicked, candidate selection falsely excludes all available accounts due to over-aggressive filtering of the global active account and non-running instances. Additionally, `switch_account_to_instance` blocks synchronously for 35+ seconds across multiple timeouts (verified close, relaunch verification, and CDP dialog dismissal).
4. Prompt backup accumulation in `backup_prompts.db` lacks a FIFO retention cap (default 100).

This subtask implements the surgical backend and frontend fixes across these subsystems.

---

## 2. Target Files & Action Breakdown

| Target File | Action & Scope |
| :--- | :--- |
| [`src-tauri/src/modules/quota/fetch_more.rs`](file:///src-tauri/src/modules/quota/fetch_more.rs) | In `fetch_quota_summary`, when `project_id.is_some()` and HTTP 403 Forbidden is returned (without `VALIDATION_REQUIRED`), retry the endpoint query with `{}` (empty JSON payload, stripping `project_id`) to fetch the user's default personal quota groups. |
| [`src-tauri/src/modules/quota/fetch_cached.rs`](file:///src-tauri/src/modules/quota/fetch_cached.rs) | Verify and ensure bucket quota fusing correctly overrides `ModelQuota.percentage` with genuine 5-hour window fraction from parsed `QuotaGroup`. |
| [`src-tauri/src/modules/auto_switcher/daemon.rs`](file:///src-tauri/src/modules/auto_switcher/daemon.rs) | After `account::fetch_quota_with_retry(&mut bound_acc)` updates `bound_acc.quota`, emit `accounts://refreshed` to the Tauri webview via `app_handle.emit("accounts://refreshed", ())`. |
| [`src-tauri/src/modules/auto_switcher/status.rs`](file:///src-tauri/src/modules/auto_switcher/status.rs) | Enforce 30s–50s polling cadence in the daemon loop whenever an active instance is running (`has_running_instances`), avoiding 120s stalls. |
| [`src-tauri/src/modules/auto_switcher/manual.rs`](file:///src-tauri/src/modules/auto_switcher/manual.rs) | Refine candidate exclusion list in `trigger_manual_rotation_for_instance`: only exclude the current target instance's bound account and accounts actively bound to *running* sibling instances. Do NOT exclude the global `current_account_id` if that instance/account is not running. |
| [`src-tauri/src/modules/auto_switcher/selection.rs`](file:///src-tauri/src/modules/auto_switcher/selection.rs) | Ensure `select_candidate_profiles` smoothly falls back to accounts above `threshold` (> 15%) when no 100% account is available, avoiding false-positive pool depletion. |
| [`src-tauri/src/modules/instance/switch.rs`](file:///src-tauri/src/modules/instance/switch.rs) | Convert synchronous 15s post-spawn relaunch verification (`verify_instance_relaunched`) and 15s CDP security notice dismissal (`dismiss_security_notice_dialog_for_data_dir`) into detached background tasks (`tauri::async_runtime::spawn`), unblocking the command return within 3s. |
| [`src-tauri/src/modules/backup_prompts_db/backup.rs`](file:///src-tauri/src/modules/backup_prompts_db/backup.rs) | Implement `pub fn enforce_max_backup_prompts_cap(conn: &Connection, max_cap: usize) -> Result<usize, String>` using strict FIFO ordering (`ORDER BY created_at DESC, id DESC LIMIT ?1`), and invoke it at the end of every prompt backup. |
| [`src-tauri/src/models/config.rs`](file:///src-tauri/src/models/config.rs) | Add `max_backup_prompts: usize` (default 100) to `AppConfig` with serde defaults. |
| [`src/pages/instances/useInstancePage.ts`](file:///src/pages/instances/useInstancePage.ts) | 1. Add `fetchAccounts()` to `performThrottledPoll()` Promise.all array.<br/>2. Add `'accounts://refreshed'` to Tauri event listener array so the frontend account store syncs immediately. |
| [`src/components/settings/AdvancedMaintenanceSection.tsx`](file:///src/components/settings/AdvancedMaintenanceSection.tsx) | Add numeric slider/input for "Max Prompt Backups Retention Cap" (range 10 to 500, default 100) bound to `config.max_backup_prompts`. |

---

## 3. Detailed Step-by-Step Implementation

### 3.1 Step 1: Quota Summary 403 Fallback in `src-tauri/src/modules/quota/fetch_more.rs`
1. Modify `fetch_quota_summary`:
   - If `project_id` is passed: prepare an initial attempt with `{ "project": pid }`.
   - If response status is `403 Forbidden` and response body does NOT contain `validation_required` or `verify your account`, log info and immediately retry with payload `{}` (without `project_id`).
   - If response succeeds with 200 OK, parse `QuotaSummaryResponse` and return the parsed `QuotaGroup` vector.
2. In `fetch_cached.rs`, verify that when `quota_groups` contains buckets with 5-hour window, `model.percentage` is updated to `(b.remaining_fraction * 100.0).round()`.

### 3.2 Step 2: Live Quota Event Emission in `src-tauri/src/modules/auto_switcher/daemon.rs`
1. In `check_and_rotate_with_options`:
   ```rust
   if let Ok(fresh_quota) = account::fetch_quota_with_retry(&mut bound_acc).await {
       bound_acc.quota = Some(fresh_quota);
       crate::error::record_ignored(account::save_account(&bound_acc), "save_account");
       if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
           use tauri::Emitter;
           crate::error::record_ignored(
               handle.emit("accounts://refreshed", ()),
               "emit accounts://refreshed",
           );
       }
   }
   ```
2. In `evaluate_and_execute_startup_rotation`, ensure the same event emission occurs upon startup quota updates.

### 3.3 Step 3: Active Daemon Polling Cadence in `src-tauri/src/modules/auto_switcher/status.rs`
1. In `start_auto_switcher()` loop:
   ```rust
   let has_running = cur_status.monitored_instances.iter().any(|i| i.is_running);
   let mut interval_secs = calculate_next_interval_seconds(lowest_monitored_quota, &switcher_cfg) as u64;
   if has_running {
       // When an IDE is actively running, bound quota check between 30s and 50s
       interval_secs = interval_secs.clamp(30, 50);
   }
   ```

### 3.4 Step 4: Candidate Exclusion Clean-up in `src-tauri/src/modules/auto_switcher/manual.rs`
1. In `trigger_manual_rotation_for_instance`:
   - Fetch running sibling instances:
     ```rust
     let registry = instance::load_registry()?;
     let mut excluded: Vec<String> = Vec::new();
     if let Some(ref curr) = current_bound {
         excluded.push(curr.clone());
         if let Ok(acc) = account::load_account(curr) {
             excluded.push(acc.email.clone());
         }
     }
     for inst in &registry.instances {
         if inst.id != inst_id && instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
             if let Some(ref acc_id) = inst.bound_account_id {
                 excluded.push(acc_id.clone());
                 if let Ok(acc) = account::load_account(acc_id) {
                     excluded.push(acc.email.clone());
                 }
             }
         }
     }
     ```
   - Do NOT append `get_current_account()` unconditionally if that account belongs to a stopped instance.

### 3.5 Step 5: Non-Blocking Async Relaunch in `src-tauri/src/modules/instance/switch.rs`
1. In `switch_account_to_instance`:
   - After `start_antigravity_with_fallback_path` returns `Ok`, do NOT block for 15s in `verify_instance_relaunched` and 15s in `dismiss_security_notice_dialog_for_data_dir`.
   - Spawn background task:
     ```rust
     let verify_inst_id = instance.id.clone();
     let verify_data_dir = instance.data_dir.clone();
     let verify_pid = instance.pid;
     let cdp_email = account.email.clone();
     tauri::async_runtime::spawn(async move {
         let _ = crate::modules::instance::verify_instance_relaunched(
             &verify_inst_id,
             &verify_data_dir,
             verify_pid,
         );
         let cdp_dir = std::path::PathBuf::from(&verify_data_dir);
         crate::modules::instance::spawn_security_notice_dismisser(
             cdp_dir,
             Some(cdp_email),
             std::time::Duration::from_secs(15),
         );
     });
     ```
   - Restore prompts and return `Ok(())` immediately.

### 3.6 Step 6: FIFO Prompt Backup Cap in `src-tauri/src/modules/backup_prompts_db/backup.rs`
1. Implement `enforce_max_backup_prompts_cap`:
   ```rust
   pub fn enforce_max_backup_prompts_cap(
       conn: &rusqlite::Connection,
       max_cap: usize,
   ) -> Result<usize, String> {
       if max_cap == 0 {
           return Ok(0);
       }
       let total_count: usize = conn
           .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
           .unwrap_or(0);
       if total_count <= max_cap {
           return Ok(0);
       }
       let deleted = conn
           .execute(
               "DELETE FROM prompt_backups 
                WHERE id NOT IN (
                    SELECT id FROM prompt_backups 
                    ORDER BY created_at DESC, id DESC 
                    LIMIT ?1
                )",
               rusqlite::params![max_cap as i64],
           )
           .map_err(|e| format!("Failed to prune prompt backups to cap {}: {}", max_cap, e))?;
       let _ = conn.execute(
           "DELETE FROM backup_batches 
            WHERE id NOT IN (SELECT DISTINCT backup_batch_id FROM prompt_backups)",
           [],
       );
       Ok(deleted)
   }
   ```
2. In `backup_active_running_prompts_for_instance`, load `app_config.max_backup_prompts` (defaulting to 100) and call `enforce_max_backup_prompts_cap(&conn, cap)`.

### 3.7 Step 7: Frontend Polling & Settings UI
1. In `src/pages/instances/useInstancePage.ts`:
   - In `performThrottledPoll`: add `fetchAccounts()`.
   - In Tauri listener effect: add `'accounts://refreshed'`.
2. In `src/components/settings/AdvancedMaintenanceSection.tsx`:
   - Add input for `max_backup_prompts` with range `10..=500` and default `100`.

---

## 4. Verification & Testing Protocol

1. **Unit Tests**:
   - `test_fetch_quota_summary_403_fallback`: Verify fallback to `{}` when project returns 403.
   - `test_enforce_max_backup_prompts_cap`: Seed 150 backup records into temporary SQLite db with cap = 100; assert 50 oldest deleted, 100 newest preserved in strict FIFO order.
   - `test_manual_fast_forward_candidate_exclusions`: Assert stopped instances and inactive accounts are not excluded from candidate pool.
2. **Pre-flight Checks**:
   - `cargo fmt -- --check` in `src-tauri`
   - `cargo clippy --all-targets --all-features` in `src-tauri`
   - `npm run build`

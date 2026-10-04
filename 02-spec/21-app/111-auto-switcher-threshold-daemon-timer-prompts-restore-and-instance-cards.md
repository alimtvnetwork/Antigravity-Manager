# 111 - Auto-Switcher Threshold Evaluation Fix, Daemon Next-Check Timer, 5s Prompt Restoration & Instance Card Projects

## 1. Executive Summary & Root Cause Analysis

### 1.1 Quota Threshold Switch Failure Root Cause
- **Banned Model Filter Interception**: In `src-tauri/src/modules/auto_switcher.rs` (lines 123, 135, 267, 282, 664, 721, 779), an outdated hardcoded filter:
  ```rust
  let is_banned = name_lower.contains("3.0") || name_lower.contains("3.1");
  ```
  categorically disqualified `Gemini 3.1 Pro (High)` from quota evaluation. When an instance consumed `Gemini 3.1 Pro` down to 6%, loop 2 (`check any model <= threshold`) skipped it because `is_banned == true`.
- **Fallthrough Distortion**: When `matched_model` was `None`, evaluation fell back to averaging non-depleted models (100%), producing a bogus quota score of 80%–100%. `should_rotate` evaluated to `false`, silently blocking automated account switching.
- **Candidate Account Elimination**: Candidate rotation scoring and weekly allowance algorithms also executed this ban filter, preventing accounts healthy on `3.1 Pro` from being chosen.
- **Interval Clamp Restriction**: `MIN_GOOGLE_QUOTA_CHECK_SECONDS = 120` clamped caution (60s) and critical (40s) intervals to 120s, suppressing rapid evaluation when credits were nearing exhaustion.

### 1.2 Missing Daemon Next-Check Telemetry
- The auto-switcher background loop ran silently without storing `next_check_timestamp` in `RUNTIME_STATE`.
- The UI had zero visibility into whether the daemon was ticking, in caution/critical mode, or how many seconds remained before the next evaluation.

### 1.3 Prompt Backup & 5-Second Delayed Restoration
- Previously, `switch_account_to_instance` in `instance.rs` and `auto_switcher.rs` blocked synchronously with nested 7-second sleeps (14s total), freezing UI feedback.
- Prompt backup before switch and prompt restoration post-launch must be strictly asynchronous:
  1. Backup running and enqueued prompts from SQLite and write `.antigravity_resume_task.json`.
  2. Switch credentials and launch IDE process.
  3. Spawn an asynchronous background task with an exact 5-second timer (`tokio::time::sleep(Duration::from_secs(5))`) that injects `.antigravity_resume_task.json` and notifies the running instance.

### 1.4 Instance Card Recent / Running Projects & Sizing Mode
- Users running multiple instances need to inspect which projects are active or recently touched directly on the instance card (up to 1–3 projects, configurable via Settings).
- Double-clicking any project chip immediately opens `PromptTreeViewModal` auto-focused and expanded on that project.
- A card density toggle (`Normal Cards` vs `Compact / Small Cards`) allows monitoring more instances simultaneously.

---

## 2. Technical Specification

### 2.1 Backend Auto-Switcher Enhancements (`src-tauri/src/modules/auto_switcher.rs`)
1. **Eliminate Bogus Model Bans**:
   - Remove `contains("3.1")` and `contains("3.0")` bans across `evaluate_account_period_status`, `calculate_account_quota`, `calculate_4h_window_quota`, `calculate_weekly_window_quota`, and `score_candidate_account`.
   - In `evaluate_account_period_status`:
     - Evaluate the configured `target_model` first.
     - Also evaluate the actively consumed models and find the lowest remaining quota across models with nonzero limits. If ANY actively used model or primary model is `<= threshold_percent`, mark as low quota and trigger rotation!
2. **Daemon Status Telemetry & Live Countdown**:
   - Extend `AutoSwitcherRuntimeState` with:
     ```rust
     pub next_check_timestamp: i64,
     pub check_interval_seconds: u32,
     pub current_stage: String, // "normal" | "caution" | "critical"
     ```
   - Expose `get_auto_switcher_daemon_status` Tauri command returning `AutoSwitcherDaemonStatus`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize)]
     pub struct AutoSwitcherDaemonStatus {
         pub is_daemon_running: bool,
         pub last_evaluated_at: i64,
         pub next_check_timestamp: i64,
         pub next_check_in_seconds: i64,
         pub check_interval_seconds: u32,
         pub current_stage: String,
         pub active_account_email: Option<String>,
         pub current_quota_percent: f64,
         pub monitored_instance_count: usize,
     }
     ```
   - In daemon loop, emit `auto-switcher://status-tick` every second with countdown and stage.
   - Adjust `calculate_next_interval_seconds` to allow caution (down to 30s) and critical (down to 15s) intervals without hard 120s clamping when explicitly configured.

### 2.2 Asynchronous 5-Second Delayed Prompt Restoration
1. In `src-tauri/src/modules/instance.rs` and `auto_switcher.rs`:
   - Prior to killing/switching an instance, call `crate::modules::repo_db::backup_running_prompts(&inst.id)` and `backup_prompts_db::backup_active_running_prompts(&inst.id)`.
   - Update account credentials in SQLite and write instance config.
   - Launch IDE instance process and immediately release the caller.
   - Spawn an async task:
     ```rust
     let target_inst_id = inst.id.clone();
     let workspace_roots = get_instance_workspace_paths(&target_inst_id);
     tauri::async_runtime::spawn(async move {
         tokio::time::sleep(std::time::Duration::from_secs(5)).await;
         log::info!("[PromptRestore] 5s post-launch delay elapsed. Restoring prompts for instance {}", target_inst_id);
         let _ = crate::modules::repo_db::restore_and_inject_prompts_for_instance(&target_inst_id, &workspace_roots);
     });
     ```

### 2.3 Local Isolated End-to-End Test (`src-tauri/tests/auto_switcher_e2e_test.rs`)
- Author a dedicated integration test covering:
  1. Low quota detection on `Gemini 3.1 Pro` at 6% triggering rotation when threshold is 15%.
  2. Prompt backup snapshotting into `.antigravity_resume_task.json`.
  3. 5-second asynchronous restoration verification.
- Strictly marked `#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]` so CI and routine local builds never run it.

### 2.4 Frontend Instance Cards & Settings Enhancements
1. **Instance Cards Projects Section (`src/pages/Instances.tsx`)**:
   - Query `get_project_conversation_tree` (`onlyRunning: false, maxWords: 50`) and match projects to instances.
   - Render up to `maxProjects` (from config, default 3) below executable path:
     - Project name chip with folder icon.
     - `RUNNING` badge if active tasks exist.
     - Total turn count badge.
     - Double click handler opens `PromptTreeViewModal` with `initialSelectedProjectId`.
2. **Card Density Toggle**:
   - Segmented pill in toolbar: `[Normal Cards]` (`LayoutGrid`) vs `[Compact Cards]` (`Grid3X3`).
   - Compact mode uses tighter grid (`grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5`) and reduced padding.
3. **Daemon Next-Check Countdown Display**:
   - In Instances page toolbar: Live countdown pill `[⏱ 42s Next Check]`.
   - In Settings Auto Profile Switcher card: Animated pulse dot with `Next evaluation in {remainingSec}s`.
4. **Settings Configuration**:
   - Add `instance_card_max_projects` (1–3, default 3) in Settings and `AppConfig`.
5. **Modern Fluid Progress Bar**:
   - Verify `WaterDrainProgressBar` renders `#1af18d` to `#12b27d` gradient, milestone checkmarks, and low/caution/critical colors without excess padding.

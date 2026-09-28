# 01 — 4-Part RCA: Account Switch Missing IDE Close/Reopen, Unnormalized `+1000` Candidate Scores, and Stale Binding Lockouts

## Part 1 — Symptom & Observed Failure
1. **Missing IDE Close & Reopen on Switch**: Clicking the Switch button (`⇄`), Fast-Forward button (`⏩`), or triggering Auto-Switch failed to reliably close the running `Antigravity.exe` IDE before writing credentials and failed to reopen the IDE and re-inject backed-up prompts in strict order.
2. **Duplicate / Divergent Switch Pipelines**: Fast-Forward (`useInstanceStore.smartRotateProfileAccount`) and Auto-Switch (`auto_switcher::execute_profile_rotation_with_context`) duplicated parts of the switch lifecycle instead of delegating to the unified Switch button pipeline (`1. Backup Running Prompts -> 2. Close Antigravity IDE -> 3. Switch Account Credentials -> 4. Re-Open Antigravity IDE -> 5. Re-Inject Running Prompts`).
3. **Inflated Candidate Scores (`+ 1000` / `+ 100000`)**: Candidate scoring in `src/services/instanceService.ts` and `src-tauri/src/modules/auto_switcher.rs` produced large unnormalized numbers (`100500`, `500`, `+ 1000`) instead of compact scores divided by `1000` (`0.000`–`0.500` with `< 100%` quota = `0`).
4. **Indefinite Lockout on Stale Bindings**: Accounts bound to idle instances or remote Supabase workstations remained permanently excluded even after `6–10` hours of zero ping or credit consumption.

---

## Part 2 — Root Cause Analysis (Grounded in Code)
1. **Root Cause A — `ls_running` Misclassification & Broken `hot_switch` in `src-tauri/src/modules/integration.rs:259-351`**:
   - `DesktopIntegration::on_account_switch` executed:
     ```rust
     let ls_running = process::is_process_running_by_name("language_server");
     let ide_running = process::is_antigravity_running(Some("ide")) || ls_running;
     let classic_running = process::is_antigravity_running(None) && !ide_running;
     ```
   - Because `Antigravity.exe` spawns `language_server_windows_x64.exe` as a child process, `ls_running` was always `true` whenever `Antigravity.exe` was open.
   - This forced `is_ide = true` and `effective_target = Some("ide")`, which entered the `hot_switch` path that only killed `language_server`, **skipped `process::close_antigravity`**, **skipped `write_to_system_keyring`**, and **skipped restarting `Antigravity.exe`**.
2. **Root Cause B — Incomplete 5-Step Sequence in `DesktopIntegration::on_account_switch` & `instance::switch_account_to_instance`**:
   - `on_account_switch` did not call `backup_prompts_db::backup_active_running_prompts(None)` in Step 1 or `repo_db::dispatch_running_prompts` in Step 5, and `apply_account_credentials` wrote to either Keyring OR SQLite instead of both.
   - `instance::switch_account_to_instance` did not call `repo_db::resend_all_running_commands(20)` and `repo_db::dispatch_running_prompts(&instance.id)` in Step 5.
3. **Root Cause C — Score Inflation in `src/services/instanceService.ts:544` & `src-tauri/src/modules/auto_switcher.rs:616`**:
   - `calculateMultiplicativeScore` added `+ 100000` (`baseScore + 100000`) and `score_candidate_account` returned raw `0..500` instead of dividing by `1000.0`.
4. **Root Cause D — Missing Stale Timeout in `get_active_in_use_account_ids` & `workspace_lease_manager.rs`**:
   - `get_active_in_use_account_ids` and `is_account_or_email_leased_by_other` did not expire bindings whose `last_used` / `leased_at` exceeded `stale_binding_timeout_hours` (6–10 hours) without local process activity or credit consumption.

---

## Part 3 — Concrete Remediation
1. **Enforce the 5-Step Switch Lifecycle in `integration.rs` & `instance.rs`**:
   - Remove `ls_running` from `ide_running` detection and remove the `hot_switch` bypass so `DesktopIntegration::on_account_switch` and `instance::switch_account_to_instance` always execute:
     1. **Step 1**: Backup running prompts via AGM (`repo_db::backup_running_prompts` + `backup_prompts_db::backup_active_running_prompts`).
     2. **Step 2**: Close the running Antigravity IDE (`process::close_antigravity` / `instance::close_instance`) and wait for process exit.
     3. **Step 3**: Switch account credentials across OS Keyring (`write_to_system_keyring`), `state.vscdb` (`db::inject_token`), and `storage.json` (`device::write_profile`).
     4. **Step 4**: Re-open the Antigravity IDE (`start_antigravity_with_fallback_path` / `launch_instance`).
     5. **Step 5**: Re-inject running prompts (`resend_all_running_commands` + `restore_running_prompts` + `dispatch_running_prompts`).
2. **Delegate Fast-Forward & Auto-Switch to the Switch Button Pipeline**:
   - In `src/stores/useInstanceStore.ts` (`smartRotateProfileAccount`), remove pre-selection `closeInstance` and delegate the verified candidate directly to `useAccountStore.getState().switchAccount(verifiedCandidate.id, targetIdeParam)`.
3. **Normalize Candidate Scoring (`÷ 1000`)**:
   - Update `calculateMultiplicativeScore` (`src/services/instanceService.ts`) and `score_candidate_account` (`src-tauri/src/modules/auto_switcher.rs`) to divide by `1000` (`(activeFactor * tierMultiplier * weeklyQuotaPercent) / 1000`) and return `0` whenever 4-hour quota `< 100%`.
4. **Add Configurable `stale_binding_timeout_hours` (Default `6` Hours, Configurable `6–10` Hours)**:
   - Add `stale_binding_timeout_hours: u32` to `AutoProfileSwitcherConfig` and release any instance binding or remote lease with no process/ping/credit activity for `> stale_binding_timeout_hours * 3600` seconds.

---

## Part 4 — Verification & Regression Prevention
- Unit tests in `src-tauri/src/modules/auto_switcher.rs` and `src-tauri/src/modules/integration.rs` verifying:
  - Normalized candidate score math (`0.5`, `0.3`, `0.1`, and `0.0` for `< 100%` quota).
  - Stale binding expiration after `stale_binding_timeout_hours` (`6` hours default).
  - `resolve_effective_target` never misclassifying classic `Antigravity.exe` as `"ide"`.
- Full Rust quality gate (`cargo clippy --all-targets --all-features`) and frontend build (`npm run build`).

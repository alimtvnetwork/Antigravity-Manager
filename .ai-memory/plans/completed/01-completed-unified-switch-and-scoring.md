# Completed Plan 01 — Unified 5-Step Switch Lifecycle, Normalized `/ 1000` Candidate Scoring, and Stale Binding Timeout

- **Status**: `COMPLETED`
- **Completed At**: `2026-09-28`
- **Canonical Spec**: [01-unified-switch-fast-forward-autoswitch-spec.md](../../02-spec/21-app/01-unified-switch-fast-forward-autoswitch-spec.md)
- **4-Part RCA**: [01-rca-switch-close-reopen-and-scoring-normalization.md](../../02-spec/22-app-issues/01-rca-switch-close-reopen-and-scoring-normalization.md)

## Verified Outcomes
1. **Normalized Candidate Scoring (`÷ 1000`, `< 100%` Quota = `0`)**:
   - Replaced `+ 1000` / `+ 100000` score inflation in `src/services/instanceService.ts` (`calculateMultiplicativeScore`, `findBestSmartPlayAccount`) and `src-tauri/src/modules/auto_switcher.rs` (`score_candidate_account`) with `(S_active * M_tier * Q_weekly) / 1000.0` (`0.000`–`0.500`), where any account with `< 100%` 4-hour rolling quota receives `0` (`0.0`).
2. **Stale / Inactive Binding & Supabase Lease Expiration (`6h`–`10h` Configurable & Zero-Credit Eviction)**:
   - Added `stale_binding_timeout_hours: u32` (default `6` hours, configurable `6`–`10`+ hours) to `AutoProfileSwitcherConfig` in `src-tauri/src/models/config.rs` and `src/services/instanceService.ts`.
   - Updated `get_active_in_use_account_ids()`, `is_account_or_email_leased_by_other()`, and `useInstanceStore.ts` so bindings/leases with no ping, process activity, or credit loss for `> stale_binding_timeout_hours` — or with `0%` credits — are automatically treated as inactive and released.
3. **Unified 5-Step Switch Lifecycle Across Switch Button, Fast-Forward Button, and Auto-Switch**:
   - Fixed `ls_running` misclassification in `src-tauri/src/modules/integration.rs` and enforced the strict 5-step sequence in `DesktopIntegration::on_account_switch` and `instance::switch_account_to_instance`:
     1. **Step 1**: Backup running prompts via AGM (`repo_db::backup_running_prompts` + `backup_prompts_db::backup_active_running_prompts`).
     2. **Step 2**: Close the running Antigravity IDE (`process::close_antigravity` / `instance::close_instance`).
     3. **Step 3**: Switch account credentials (`write_to_system_keyring` + `state.vscdb` + `storage.json` + instance binding).
     4. **Step 4**: Re-open the Antigravity IDE (`start_antigravity_with_fallback_path` / `launch_instance`).
     5. **Step 5**: Re-inject running prompts (`resend_all_running_commands` + `restore_running_prompts` + `dispatch_running_prompts`).
   - Updated `useInstanceStore.smartRotateProfileAccount` and `auto_switcher::execute_profile_rotation_with_context` to delegate directly to the unified Switch button pipeline.
4. **AGM & Telegram `/tree`, `/prompt C001`, `/ssh`, `/update` Parity**:
   - Added `agm_project_sequences` (`P001..`) and `agm_conversation_sequences` (`C001..`) in `repo_db.rs`, `/tree`, `/ssh`, `/update` in `telegram_inbound.rs`, `agm tree`/`agm agy`/`agm update` in `src-tauri/src/bin/agm.rs`, and updated `EmailNotificationSettings.tsx`.

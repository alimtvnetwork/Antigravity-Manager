# Plan 47: CLI & Auto-Switcher Deep Verification Remediation (Completed)

**Status:** Completed
**Completed:** 2026-09-26
**Spec Reference:** `02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md`, `02-spec/21-app/46-cli-expansion-auto-switch-and-email-telemetry.md`

## Summary of Verified Outcomes

1. **Auto-Switcher Minimum-Bottleneck Quota & Reactive 5-Second Loop (`src-tauri/src/modules/auto_switcher.rs`)**:
   - Updated `calculate_account_quota` and `evaluate_account_period_status` to compute the minimum percentage across matching target models, non-banned flash models, actively consumed non-banned models (`percentage < 100` / `<= threshold_percent`), and `quota_data.quota_groups` buckets.
   - Updated `select_next_best_profile` to treat unqueried standby accounts as `100.0%` (`unwrap_or(100.0)`) and fall back to the highest-scoring candidate (`quota > 15.0`) when high-threshold testing (e.g. `98%`) finds no candidates above `threshold`.
   - Updated `start_auto_switcher` to execute an initial check after startup and sleep in 5-second ticks, waking up immediately whenever `is_enabled` is toggled on or `low_quota_threshold_percent` is modified in the UI.
2. **Switch Email `[JSON]` Telemetry State Machine (`src-tauri/src/modules/notification_hub.rs` & `src-tauri/src/modules/instance.rs`)**:
   - Added `record_previous_email` (`PREVIOUS_EMAIL_STATE`) and 5-second deduplication (`LAST_SWITCH_DISPATCH`) in `notification_hub.rs`.
   - Captured `prev_email` in `instance::switch_account_to_instance` prior to credential binding.
   - Populated `"old_email"`, `"new_email"`, `"instance_id"`, `"instance_name"`, `"instance_mode"` (`"default"` vs `"isolated"`), `"switch_mode"`, `"condition"`, `"reason"`, `"agm_version"`, `"vm_name"`, and `"local_ip"` in `[JSON]` switch notification emails.
3. **CLI Email Dispatch, Instance Data-Only Clone & Project Recreate (`src-tauri/src/bin/agm.rs`)**:
   - Updated `agm email status` and `agm email help` to dispatch HTML status/help emails (`[Antigravity | vX.Y.Z | VM | IP] [Antigravity] [JSON] Node & Credits Status` and remote instructions cheat-sheet) to configured recipients in addition to terminal output.
   - Updated `agm instances create "new-name" [--data-only | --do | do]` to clone user data/settings from `"default"` via `instance::copy_instance("default", name, Some("full"))` and clone the IDE binary unless `--data-only` (`do`) is passed.
   - Updated `agm recreate-project` and `agm recreate <seq|id|alias|path>, ...` to support comma-separated targets, auto-detect git repo root, purge matching `workspaceStorage` folders, conversation `.db` files, and `brain/<cid>` directories, and seed `"read all files and memory to understand the project"` in `repo_prompts.db` and `.antigravity_resume_task.json` before launching `agy`.

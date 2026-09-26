# PLAN-046: CLI Expansion, Auto-Switch If Low Credit & Email Multi-VM Telemetry State Machine

- **Plan ID**: `PLAN-046`
- **Status**: `COMPLETED`
- **Specification Reference**: [`02-spec/21-app/46-cli-expansion-auto-switch-and-email-telemetry.md`](../../../02-spec/21-app/46-cli-expansion-auto-switch-and-email-telemetry.md)

---

## 1. Consolidated Summary of Completed Subtasks

### Subtask 01: Auto-Switch If Low Credit Engine & RCA Fix
- **Root Cause Resolved**: `src-tauri/src/modules/auto_switcher.rs` previously skipped the `default` instance when `inst.bound_account_id` was `None` in `instances.json`, and relied on cached quota values without refreshing from the Google API before evaluating thresholds.
- **Remediation**:
  - Updated `get_active_in_use_account_ids()`, `get_status()`, `trigger_manual_rotation_for_instance()`, and `check_and_rotate_with_options()` to fall back to `account::get_current_account_id().ok()` whenever `inst.bound_account_id` is `None`.
  - Added live quota refresh (`account::fetch_quota_with_retry(&mut bound_acc).await`) before evaluating low-quota and critical-quota thresholds.
  - Exported `check_and_rotate_for_threshold(custom_threshold: Option<f64>, force: bool)` supporting custom thresholds up to `99.0%` (e.g. `98%` for immediate validation).
  - Updated `src/components/settings/AutoSwitcherSettings.tsx` slider and clamp to allow thresholds up to `99%`.

### Subtask 02: Email Subject Standardization & Multi-VM Telemetry State Machine
- **Canonical Subject Format**: Updated `src-tauri/src/modules/email_sender.rs` (`format_subject_with_telemetry` and all email renderers) to emit `[Antigravity | v{VERSION} | {VM_NAME} | {LOCAL_IP}] [Antigravity] <Subject>`.
- **Multi-VM JSON State Machine**:
  - Updated `dispatch_email_switch_alert` in `src-tauri/src/modules/notification_hub.rs` to tag subjects with `[JSON]` (`[Antigravity | v{VERSION} | {VM_NAME} | {LOCAL_IP}] [Antigravity] [JSON] Account Switched: {instance_name} -> {account_email}`) and embed a structured JSON state payload in the email body.
  - Added `fetch_recent_cross_vm_switched_accounts(lookback_seconds: i64)` in `src-tauri/src/modules/email_inbound.rs` and wired it into `auto_switcher::select_next_best_profile` so sibling VMs exclude accounts recently switched to by other VMs.
- **Self-Email on Config Addition**:
  - Added `notify_email_config_added` in `src-tauri/src/modules/notification_hub.rs` and wired it into `src-tauri/src/commands/email.rs` (`add_email_account`, `update_email_account`, `add_notify_recipient`) and `agm email add`.

### Subtask 03: CLI Prompt Inspection, Export/Import & Rerun Suite
- Implemented in `src-tauri/src/bin/agm.rs`:
  - `agm status` / `agm credits` (`[--json]`, immediate & weekly quotas).
  - `agm switch-if-low-credit` (aliases: `swlc`, `sfc`, `switch-if-no-credit`).
  - `agm which-prompts-running` (alias: `wpr`) `[--json]` yielding `<seq>`, project, ID, conv ID, conv name, and prompt queue count.
  - `agm prompts ls [N] [--json] [--words 100]` displaying running prompts in ASC stack order and explicitly noting table mode vs `--json`.
  - `agm prompts-export` (`pe`) `[N] [-f <path>]` defaulting to `agm-<repo-slug>-prompts.json` with Base64 image encoding.
  - `agm prompts-import` (`pi`) `[-f <path>]` with interactive multi-JSON discovery and prompt rerun queueing.
  - `agm prompt "<text>" [--prefix <cat>] [--suffix <cat>]` and `agm rerun [prompts [N]] [-prefix <cat>]` with automatic `git pull` and `01-prompts/` template resolution.

### Subtask 04: CLI Instances, Cache Clear & Email Vault Commands
- Implemented in `src-tauri/src/bin/agm.rs`:
  - `agm clear-cache` / `cache-clear` / `clear cache` `[--keep/-k 10]` calling `agy_cleaner::prune_and_clean`.
  - `agm instances`: `ls [--json]`, `<seq|id|alias> [switch] ff`, `instances-all ff`, `rm <seq|id|alias>`, `create "<name>" [--data-only|--do]`, and `rm-all` (strictly preserving `default`).
  - `agm email`: `status [--json]`, `help`, `ls [--json]`, `add`, `rm`, `mv <seq|id|email> --default`, `export`, and `import`.

### Subtask 05: CLI Project Recreation (`recreate-project` & `recreate`)
- Implemented `cmd_recreate_project` (`agm recreate-project [path]`) and `cmd_recreate` (`agm recreate [targets...]`) in `src-tauri/src/bin/agm.rs` to purge cached `workspaceStorage` folders, clean `repo_prompts.db` state, remove stale `.antigravity_resume_task.json` files, and launch a fresh Antigravity IDE session.

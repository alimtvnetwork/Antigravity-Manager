# 48: UI Smart Fast-Forward Auto-Switcher Hook, Default Switch Telemetry & Instance Hardening

**Status:** Completed
**Completed Date:** 2026-09-26
**Specification:** [`02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md`](../../../02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md)

## Summary of Completed Tasks

- [x] **Subtask 01: Backend Auto-Switcher, Instance & Default Switch Telemetry Hardening**
  - **Live Quota Persistence**: In `src-tauri/src/modules/auto_switcher.rs`, immediately persist live account quota to disk via `account::save_account(&bound_acc)` after successful API quota refresh.
  - **Default Instance Switch Telemetry**: In `src-tauri/src/commands/mod.rs` (`switch_account`), captured `prev_email` before switch via `notification_hub::record_previous_email` and dispatched `notification_hub::notify_account_switched` on default instance switches (Smart Fast-Forward `>>`).
  - **Immediate Check on Config Save**: In `src-tauri/src/commands/mod.rs` (`save_config`) and `src-tauri/src/commands/instance.rs` (`update_auto_switcher_config`), spawn immediate `check_and_rotate_if_needed` when `auto_profile_switcher.is_enabled` is true.
  - **Isolated Instance Profile Storage**: In `src-tauri/src/modules/instance.rs` (`switch_account_to_instance`), write `device_profile` into `<instance_data_dir>/User/globalStorage/storage.json` for isolated instances.
  - **Instance Creation Settings Seeding**: In `src-tauri/src/modules/instance.rs` (`create_instance`), seeded `User/settings.json` from the default installation.
  - **1-Based Index & Error Handling**: In `src-tauri/src/modules/instance.rs` (`resolve_instance_id`), resolved 1-based index `#1`, `#2` from `registry.instances` and returned `Err` on unmatched specifiers instead of silently falling back to the active instance.

- [x] **Subtask 02: Frontend BackgroundTaskRunner Smart Fast-Forward Trigger & CLI Flag Polish**
  - **Frontend Auto-Switcher Hook**: In `src/components/common/BackgroundTaskRunner.tsx`, added reactive `useEffect` watching `config?.auto_profile_switcher` that computes effective quota and triggers `smartRotateProfileAccount(activeInstanceId)` (UI Smart Fast-Forward `>>`), refreshes UI stores, and shows toast notifications immediately upon threshold slider adjustment (`98%`) and interval ticks.
  - **CLI Flag Alias**: In `src-tauri/src/bin/agm.rs` (`cmd_clear_cache`), added support for `--keep/k`, `-keep/k`, `keep/k`, and `--keep/k=10` flag syntax.
  - **Email Subject Standardization**: Standardized all email subjects across `email_sender.rs` to start with `[Antigravity | v{} | {} | {}] [Antigravity] ...`.

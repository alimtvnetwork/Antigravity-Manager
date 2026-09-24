# Plan 69: Instance Delete & Open/Close Loop Fix, Auto-Switcher 5m/15%/12% Ladder, Navbar Dropdown Consolidation & 100% Plaintext Subject-Driven Email Engine (COMPLETED)

- **Status**: `completed`
- **Completed At**: `2026-09-24`
- **Target Version**: `v4.70.0`
- **Spec Reference**: `02-spec/21-app/25-instance-delete-auto-switcher-navbar-and-plaintext-email-fixes.md`
- **RCA Reference**: `02-spec/22-app-issues/04-instance-delete-open-close-loop-and-focus-stealing-rca.md`

---

## Consolidated Subtasks & Verified Outcomes

1. **Subtask 01 (`01-instance-delete-and-open-close-loop-fix.md`) — COMPLETED**:
   - Fixed `delete_instance` in `src-tauri/src/modules/instance.rs` to automatically call `close_instance(instance_id)` first, reset `active_instance_id` to `"default"` when active, remove the instance from `instances.json`/`instances.db` first, and resiliently delete `instance_folder` and any legacy `Antigravity-<id>.exe`.
   - Fixed `launch_instance` in `src-tauri/src/modules/instance.rs` to use the standard `Antigravity.exe` with `--user-data-dir=<data_dir>` instead of `clone_instance_executable`.
   - Made `check_and_recover_crashed_instance()` in `src-tauri/src/modules/auto_switcher.rs` passive so it never calls `trigger_manual_rotation()` or spawns windows in the background.

2. **Subtask 02 (`02-release-workflow-split-codeblocks-and-v4690-fix.md`) — COMPLETED**:
   - Updated `.github/workflows/release.yml` to generate 4 separate copy-pasteable code blocks without `# Or pinned version:`.
   - Updated the live `v4.69.0` GitHub Release via `gh release edit v4.69.0`.

3. **Subtask 03 (`03-auto-switcher-dynamic-intervals-gemini38-and-email.md`) — COMPLETED**:
   - Updated `AutoProfileSwitcherConfig` defaults in `src-tauri/src/models/config.rs`, `src-tauri/src/modules/config.rs`, and `src/components/settings/AutoSwitcherSettings.tsx` to `check_interval_seconds = 300` (5m), `low_quota_threshold_percent = 15.0` (`< 15%` -> `60s`), `critical_threshold_percent = 12.0` (`<= 12%` -> `40s`), `target_model = "gemini-3.8-flash-high"` (`Gemini 3.8 Flash High (Primary, Recommended)`), and `auto_focus_window = false`.
   - Updated `calculate_next_interval_seconds` and `calculate_account_quota` in `src-tauri/src/modules/auto_switcher.rs` plus unit tests.

4. **Subtask 04 (`04-navbar-dropdown-collision-debug-left-and-toolbar-consolidation.md`) — COMPLETED**:
   - Added `agm:dropdown-open` custom event coordination across `NavMenu.tsx`, `InstanceSelector.tsx`, and `NavSettings.tsx` so opening `Accounts` closes `Instances` and vice-versa.
   - Moved the Debug (`Bug`) button from `NavSettings.tsx` to the left brand area in `Navbar.tsx`.
   - Combined Theme + Language into a single compact dropdown in `NavSettings.tsx`, leaving only the `Quick Clean` (Recycle) button beside it.
   - Combined `Import` and `Export` into a single icon dropdown button (`Import / Export`) in `InstanceSelector.tsx` and `Accounts.tsx`, with `z-[9999]` overlays.

5. **Subtask 05 (`05-window-focus-stealing-rca-and-fix.md`) — COMPLETED**:
   - Defaulted `auto_focus_window` to `false` and removed background `trigger_manual_rotation()` window launches from `check_and_recover_crashed_instance()` so Antigravity IDE never steals focus while the user is working in Windows Explorer.

6. **Subtask 06 (`06-plaintext-email-subject-commands-and-telegram-ps-alignment.md`) — COMPLETED**:
   - Replaced all `<div style=...>` HTML strings in `src-tauri/src/commands/email.rs` (`dispatch_custom_email_task`) with 100% plaintext ASCII where the command is in the `Subject` (`* | prompt | proj-Antigravity-Manager` or `* | ps | Get-Process`) and ONLY the pure prompt/command is in the `Body`.
   - Removed `powershell:` prefixes from `EmailNotificationSettings.tsx` and `ai-sample-templates-modal.tsx`.
   - Fixed horizontal alignment (`items-end`, `h-5` flex labels) between `Telegram Bot Token` and `Allowed Chat ID (Numeric)` in `EmailNotificationSettings.tsx` and created `03-ai-scripts/telegram-bot-helper.ps1`.

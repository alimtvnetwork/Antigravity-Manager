# Plan 64: Account Switch E2E Verification, Parallel Prompt Backup, Multi-VM Collision Prevention & 15% Threshold Release

Spec Reference: [02-spec/21-app/64-account-switch-e2e-prompt-backup-verification.md](../../../02-spec/21-app/64-account-switch-e2e-prompt-backup-verification.md)

## Summary & Verification
Completed end-to-end account switching under high threshold (98%), parallel prompt backup to SQLite DB, Supabase & Email multi-VM lease collision avoidance, Telegram and Email telemetry detailing human-readable project names (no raw UUIDs or hashes), polished CLI help text with copy-pasteable examples, aligned default threshold to 15.0%, verified default and isolated sandbox instance mode, and performed minor bump release.

Total Loops / Steps: 6 execution steps across Phase 1 and Phase 2.

## Consolidated Subtasks

### Subtask 01: CLI Help Text & Syntax Examples Polish
- Enhanced `print_help()` and command-specific help in `agm.rs` (`backup`, `restore`, `auto-switch`, `switch-if-low-credit`).
- Provided structured options, aliases, and concrete copy-pasteable usage examples.
- Verified: `agm --help`, `agm backup --help`, `agm restore --help`, `agm switch-if-low-credit --help` render structured syntax and examples.

### Subtask 02: Parallel Prompt Backup & Project Name Mapping
- Implemented multi-project parallel prompt scanning via `std::thread::scope` in `repo_db.rs`.
- Added `resolve_friendly_project_name()` in `backup_prompts_db.rs` to extract human-readable project directory basenames instead of raw UUIDs.
- Synchronized image payloads and attached disk paths into `.antigravity_resume_task.json`.
- Implemented and exposed `verify_prompts_running()` returning active running prompt count.

### Subtask 03: Multi-VM Account Lease & Collision Prevention
- Implemented `is_account_or_email_leased_by_other(account_id, email)` in `workspace_lease_manager.rs`.
- Integrated lease checks in `auto_switcher.rs` candidate selection for instances, global pool accounts, and pre-switch verification.
- Verified cross-VM collision prevention via Supabase root DB and recent IMAP switch telemetry (`fetch_recent_cross_vm_switched_accounts`).

### Subtask 04: Telegram & Email Telemetry Notification of Backup & Switch Stages
- Added `backed_up_projects`, `backed_up_prompts_count`, and `restored_prompts_count` to `SwitchNotificationDetails`.
- Formatted status cards in `notification_hub.rs` (`dispatch_telegram_switch_alert`, `dispatch_email_switch_alert`, `dispatch_self_json_in_use_broadcast`).
- Updated Telegram bot commands (`/ff`, `/rotate`, `/backup`, `/restore`) in `telegram_inbound.rs` to display friendly project names.

### Subtask 05: End-to-End Test (98% Simulation) & 15% Default Threshold Alignment
- Executed 98% rotation simulation: `agm switch-if-low-credit -t 98 --json` (rotated to freshest candidate with prompt preservation and re-injection).
- Created and tested isolated sandbox instance `test-sandbox-6237` via `agm instances test-sandbox-6237 ff`, then cleanly removed via `agm instances rm test-sandbox-6237 --force`.
- Standardized default quota threshold to 15.0% (`low_quota_threshold_percent: 15.0`, `critical_threshold_percent: 15.0`, `threshold_percentage: 15`) across `models/config.rs`, `modules/config.rs`, `modules/auto_switcher.rs`, `bin/agm.rs`, and `Settings.tsx`.

### Subtask 06: Minor Version Bump & Release Ceremony & CI/CD Verification
- Version bump executed via `npm run bump minor` to `v4.86.0`.
- Verified `cargo fmt` and `cargo clippy`.
- Sole maintainer attribution: `alim, devorg.bd@gmail.com, @aukgit`.
- Pipeline verified via `gitmap pe -t` to 100% green.

# Plan 63: Fast-Forward Prompt Backup, Post-Switch Re-Injection & 12% Quota Threshold

## Objective
Implement end-to-end prompt preservation during fast-forward / profile rotation, align the default quota threshold to 12%, and ensure CLI and Telegram command parity.

## Subtasks & Verification
1. `subtask-01-threshold-defaults.md`: [COMPLETED] Default threshold aligned to 12.0% in `config.rs`, `Settings.tsx`, `config.rs` migration, and `agm.rs`.
2. `subtask-02-autoswitcher-backup-reinject.md`: [COMPLETED] Pre-switch prompt backup and post-switch re-injection integrated into `auto_switcher.rs::execute_profile_rotation_with_context`.
3. `subtask-03-telegram-ff-and-help.md`: [COMPLETED] Telegram `/ff` and `ff:` updated to perform backup, rotation, and re-injection; `/help` manual catalog updated with `/help` entry and safe chunking.
4. `subtask-04-cli-backup-restore-parity.md`: [COMPLETED] `agm backup` (alias `backpack`), `agm restore`, and `agm prompt backup/restore` added to `agm.rs`.
5. `subtask-05-lint-clippy-verification.md`: [COMPLETED] `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build` all passed with code 0.

# Completed Plan: Running Prompts Split SQLite Backup/Restore, FPUG/SUG Green Watchers, Broadcast Email & Telegram CLI

Canonical Specification Reference: [50-running-prompts-backup-restore-and-green-watchers.md](file:///d:/work/Antigravity-Manager/02-spec/21-app/50-running-prompts-backup-restore-and-green-watchers.md)

## Status: Fully Completed & Verified ✅

### Subtask Execution & Verification Summary
1. **Subtask 01 - Auto-Switch Threshold 25%**:
   - Default quota threshold verified at `25.0%` in `src-tauri/src/models/config.rs`.
   - CLI command `agm auto-switch threshold [N]` implemented with typo tolerance `auto-swtich thresehold [N]`.
   - Verified via: `agm.exe auto-switch threshold` -> `Current auto-switch low quota threshold: 25.0%`.
   - Verified alias: `agm.exe auto-swtich thresehold 25` -> `[SUCCESS] Auto-switch low quota threshold set to 25.0%`.

2. **Subtask 02 - Split SQLite Backup & Restore Engine**:
   - Created `src-tauri/src/modules/backup_prompts_db.rs` with `prompt_backups`, `backup_batches`, and `green_projects` tables.
   - Default split DB storage path at `~/.antigravity_tools/backup-prompts/backup-prompts.db`, supports custom `-f/--file`.
   - CLI commands `backup-running-prompts`, `restore-running-prompts`, `running-prompts backup/restore` with `--keep/-k`, `ls`, `clean [--force]`, `--json`.
   - Restored prompt 1-day retention auto-cleanup with single-line notice `[INFO] Marked records as restored. Will be automatically cleaned up after 1 day.`
   - Module unit tests passing (`cargo test --lib backup_prompts_db`).

3. **Subtask 03 - Running Prompts Listing, Pagination, Export/Import**:
   - Implemented `agm running-prompts ls [--limit/-l Y] [--wordcount/wc N] [--full] [--json]` with default Y=8, N=100.
   - Dual-format export & import (`agm running-prompts export/import [-f path] [--wc N]`) supporting `.db` and `.json`, re-enqueuing prompts upon import into active queue.
   - Verified table view and JSON output format with ASC sequence order.

4. **Subtask 04 - Running Projects Inspection**:
   - Implemented `agm running-projects [ls] [--json] [-file/-f <path>] [--ssh]`.
   - Outputs sequence, project name, ID, conversation ID, conversation name, prompts count (queue), status, and node info.
   - Verified JSON array and formatted table output across active workspaces.

5. **Subtask 05 - FPUG and SUG Green Watchers**:
   - Implemented `agm finish-prompts-until-green` (alias `fpug`) `[targets...] [-t 5m]` and `running-projects`.
   - Implemented `agm shutdown-until-green` (alias `sug`) `ls/help/run/add-projects/rm/agy-running-projects [-t 5m]`.
   - Cross-platform native shutdown dispatch (Windows `shutdown /s /t 60`, Ubuntu/Debian `systemctl poweroff`, macOS `osascript`).

6. **Subtask 06 - Broadcast Email & Telegram CLI**:
   - Implemented `agm broadcast-email ls/add/edit/rm/help/send-to-all/send/send help/test`.
   - Implemented `agm telegram ls/set/help/set help/ping/cmds/commands` integrated with `telegram_inbound`.

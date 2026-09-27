# Plan 75: Idle Sensor Fix, GitMap Telemetry Parity & Notification Typography (Completed)

Spec Reference: [02-spec/21-app/55-idle-sensor-fix-and-gitmap-parity.md](../../../02-spec/21-app/55-idle-sensor-fix-and-gitmap-parity.md)
Issue RCA Reference: [02-spec/22-app-issues/18-idle-sensor-false-reporting-and-telemetry-parity.md](../../../02-spec/22-app-issues/18-idle-sensor-false-reporting-and-telemetry-parity.md)
Visual Reference: [assets/screenshots/agm-idle-status-fix-01.png](../../../assets/screenshots/agm-idle-status-fix-01.png)

## Summary of Accomplishments

1. **Subtask 01: Fix Idle Detection & Process Sensor**
   - In `src-tauri/src/modules/repo_db.rs`:
     - Overhauled `is_any_prompt_actively_running()` and `get_live_project_execution_info()`.
     - Connected to `conversation_summaries.db` using read-only URI mode (`OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI`) with `?immutable=1` fallback to avoid Windows WAL locks.
     - Added RFC3339 recency filter checking if `last_modified_time` was within the last 600s or in `CASCADE_RUN_STATUS_RUNNING`.
     - Implemented normalized path comparison (`normalize_path_for_compare()`) stripping backslashes and case differences.
     - Matched projects by 8-character conversation ID prefixes (e.g. `antigravity-manager-d58c5517` with `d58c5517`).
     - Added live OS process detection via `instance::find_pids_for_data_dir()`.
   - In `src-tauri/src/modules/email_watcher.rs`:
     - Added anti-false-idle gate suppressing idle email dispatch if `is_any_prompt_actively_running()` returns true.

2. **Subtask 02: Embed GitMap Telemetry & Commands Table**
   - In `src-tauri/src/modules/git_info.rs`:
     - Added `get_git_full_hash()`, `get_repo_url()`, and `get_built_timestamp()`.
   - In `src-tauri/src/bin/agm.rs`:
     - Overhauled `print_banner()` to render GitMap-style dual-box telemetry (`agm binary` with full 40-character SHA, URL, version, branch, last release, DB path, executable path, built time; and `current workspace / node` with node alias, local IP, dispatch time).
     - Added `print_commands_table()` detailing inbound email subjects (`sub: <NODE> | proj-<id>`) and CLI equivalents.
   - In `src-tauri/src/modules/email_sender.rs`:
     - Embedded GitMap telemetry table into `wrap_html_email_card()`.

3. **Subtask 03: Projects List & Prompts Inventory**
   - In `src-tauri/src/modules/email_sender.rs`:
     - Implemented Active Workspaces table right under the sensor status box with Project Name, copyable email reply target (`sub: <NODE> | proj-<id>`), normalized path, and live status badge.
     - Implemented Usable Prompts Reference List Table (`execute-pending-tasks`, `execute-parent-task`, `ci-cd-fix`, `minor-bump`, `coding-guidelines`, `smart-test-runner`).
   - In `src-tauri/src/bin/agm.rs`:
     - Added `print_prompts_reference_table()` and wired it into bare invocation and `agm help`.

4. **Subtask 04: Enlarge Typography & Notification Styling**
   - In `src-tauri/src/modules/email_sender.rs`:
     - Base body font size enlarged to 18px (line-height 1.7).
     - Card headings enlarged to 28-32px bold.
     - Section titles enlarged to 20px uppercase bold (`letter-spacing: 0.05em`).
     - Table headers and cells enlarged to 16-17px with generous padding (`16px 20px` headers, `14px 20px` cells).
     - Command badges and prompt slugs styled with 16px monospace and prominent padding (`6px 12px`).
     - Clean status badges (`RUNNING` in emerald, `IDLE / READY` in slate).

5. **Release & Verification**
   - Minor version bumped from `4.80.0` to `4.81.0` across all 14 project locations via `scripts/bump-version.mjs`.
   - Updated `CHANGELOG.md` and `CHANGELOG_EN.md`.
   - `cargo check --lib`: passed (0 errors).
   - `cargo check --bin agm`: passed (0 errors).
   - `cargo fmt -- --check`: passed (clean).
   - Targeted unit test `test_template_rendering`: passed.

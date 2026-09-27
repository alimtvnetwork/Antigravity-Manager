# Execution Plan 52: Idle Notification Logic Fix & AGM GitMap Parity (Completed)

Spec Reference: [02-spec/21-app/52-idle-notification-and-agm-parity.md](../../02-spec/21-app/52-idle-notification-and-agm-parity.md)
RCA Reference: [02-spec/22-app-issues/16-idle-sensor-false-positive-rca.md](../../02-spec/22-app-issues/16-idle-sensor-false-positive-rca.md)

## Summary of Accomplishments
1. **Idleness Sensor Ground Truth & Anti-False Positive Guard**:
   - Analyzed why project `antigravity-manager-d58c5517` was incorrectly reported as idle in outgoing telemetry emails.
   - Discovered that `check_idle_projects_sensor` relied on `repo_db::list_backed_up_prompts()`, which only queries explicit backups (`status = 'backed_up'`). During active in-flight executions, this was empty.
   - Refactored `email_watcher.rs` and `repo_db.rs` to query live ground-truth state from `~/.gemini/antigravity/conversation_summaries.db` (`not_fully_idle != 0` and `status LIKE '%RUNNING%'`), coupled with `repo_prompts.db`.
   - Added `is_any_prompt_actively_running(&conn)` to gate the idle sensor; if any project has running conversations or queued prompts, idleness notifications are strictly suppressed.

2. **GitMap Telemetry Parity (`src-tauri/src/modules/git_info.rs`)**:
   - Added Git metadata extraction module and `build.rs` compile-time environment variables (`AGM_GIT_HASH`, `AGM_GIT_BRANCH`, `AGM_LAST_RELEASE`).
   - Integrated GitMap-style banner into AGM CLI: `Version: v4.78.0 | Commit: <hash> | Branch: <branch> | Last Release: <tag>`.
   - Embedded Git telemetry into outgoing HTML email headers with a high-contrast metadata badge block.

3. **HTML Email Template Overhaul & Typography Upgrade**:
   - Enlarged font sizing across the board (Title 28px, body 16px, table cells 14-15px, monospace code 14px).
   - Applied strict Ubuntu / Segoe UI / system-ui font stack across all email containers and cards.
   - Styled all action and target links with pure white text (`#ffffff` on `#3b82f6` / `#4f46e5` pills) to guarantee high-contrast legibility.
   - Injected interactive command cheat sheet table, live project inventory with copy-pasteable reply targets (`sub: <NODE> | proj-<id>`), and recent prompt history table into every idle alert.

4. **AGM Terminal CLI Parity**:
   - Updated `agm` default output (no arguments) and `agm help` to render the GitMap header, command table categories, live workspace status table, and recent prompts queue.
   - Updated `agm version` to output `agm vX.Y.Z (commit: <hash>, branch: <branch>, last release: <tag>)`.
   - Formatted all tables with clear columns and dividers.

5. **Pre-Flight Gates & Verification**:
   - `cargo fmt -- --check`: 100% clean.
   - `cargo clippy --bin agm --lib`: Passed with 0 errors.
   - `npm run build`: Passed cleanly in 17.98s.
   - `cargo build --bin agm`: Passed cleanly in 1m 04s.
   - `target/debug/agm.exe`: Tested and verified live output with accurate running state detection.

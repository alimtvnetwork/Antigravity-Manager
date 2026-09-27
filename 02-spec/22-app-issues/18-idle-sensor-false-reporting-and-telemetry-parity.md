# Issue RCA: False Idle Workspace Sensor Reporting & Telemetry Parity Gaps (Issue 18)

## 1. Reproduction & Symptoms

- **Reported By:** User via screenshot `https://prnt.sc/l8PHOl0X3WWo` (`assets/screenshots/agm-idle-status-fix-01.png`).
- **Symptom:** AGM dispatched an email titled `Idle Projects Notification` asserting:
  `"There are no active prompts currently running in the following active projects: - antigravity-manager-d58c5517 ..."`
- **Actual State:** Antigravity project `antigravity-manager-d58c5517` was actively executing instructions in conversation `d58c5517-d8ad-437e-ab7c-e506b0322383` with status `'CASCADE_RUN_STATUS_RUNNING'` and `not_fully_idle = 1`.
- **Additional Deficiencies Noted by User:**
  1. Missing GitMap-style build & binary telemetry in AGM (`Version`, `Commit Hash`, `Branch`, `Last Release`).
  2. Missing actionable commands table at the bottom explaining how to run tasks via email and CLI.
  3. Missing clear projects list table with project names, full repository paths, and active prompt states.
  4. Missing reference list of prompts.
  5. Fonts across email notifications and cards were excessively small and hard to read.

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Why did the system declare the workspace idle when prompts were running?
In `src-tauri/src/modules/repo_db.rs`, `is_any_prompt_actively_running()` and `get_live_project_execution_info()` inspected `conversation_summaries.db` using:
```rust
rusqlite::Connection::open_with_flags(&summaries_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
```
When Antigravity IDE (Cascade/Electron) is actively executing, SQLite WAL mode holds in-flight transactions in `conversation_summaries.db-wal`. Standard `SQLITE_OPEN_READ_ONLY` fails or returns cached/stale state if WAL recovery or write checkpoints are pending. Furthermore, path comparison in `get_live_project_execution_info()` compared raw `clean_p` against `clean_path` using strict equality on lowercased strings without normalizing Windows backslashes vs POSIX slashes, drive letter casing, or trailing separators, and failed to check conversation ID prefixes (e.g. `d58c5517`).

### 2.2 Why were live process execution states ignored?
The idle sensor relied solely on SQLite tables and never checked system process tables (`sysinfo`). Even when `Antigravity.exe` or `code.exe` (with Antigravity extension) is actively consuming CPU (>0.5% or >10s compute cycles), the watcher treated the workspace as idle if database writes lagged.

### 2.3 Why was GitMap telemetry absent from AGM output?
AGM lacked the standardized dual-section binary and repository metadata banner (`gitmap binary` and `current repo/workspace`) present in GitMap. Git hash, branch, and release versioning were partially available in `git_info.rs` but not rendered in the primary CLI overview or embedded cleanly into email notification headers.

### 2.4 Prevention & Code Fix
1. **Multi-Source Activity Invariant**:
   - Query `conversation_summaries.db` with WAL sharing (`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI` with `?immutable=1` fallback) checking `not_fully_idle != 0 OR status LIKE '%RUNNING%' OR last_modified_time` within last 600s.
   - Robust path & conversation ID normalization: compare normalized paths (forward slashes, lowercase, stripped trailing slashes) AND check if `project_id` contains the conversation ID prefix.
   - Inspect active Antigravity process CPU using `instance::find_pids_for_data_dir()` and `sysinfo`. If any Antigravity process is actively running with compute activity, do NOT declare idle.
   - Suppress idle email alerts completely if `is_any_prompt_actively_running()` is true.
2. **GitMap Telemetry Parity**:
   - Embed full GitMap-style metadata in `agm` CLI banner and all outgoing email notifications: `Version`, `Commit SHA`, `Branch`, `Last Release`, and node details.
3. **Actionable Commands Table & Projects List**:
   - Render a high-visibility commands table (Send Prompt, Broadcast, Status, Auto-Switch, Prompts LS).
   - Render a structured active projects table with Project Name, Email Target ID (`proj-<id>`), Repo Path, and Status (`RUNNING` / `IDLE`).
   - List available prompt templates and instructions.
4. **Enlarged Typography**:
   - Drastically increase font sizes across email templates: body to 18px (line-height 1.7), headings to 28-32px, tables/code to 16-17px.

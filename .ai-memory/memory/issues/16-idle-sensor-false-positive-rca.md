# Issue RCA: False Positive Idle Workspace Sensor Notification (Issue 39)

## 1. Reproduction & Symptoms
- **Reported By:** User via screenshot `https://prnt.sc/l8PHOl0X3WWo` (`assets/screenshots/idle-email-notification.png`).
- **Symptom:** AGM dispatched an email titled `Idle Projects Notification` asserting:
  `"There are no active prompts currently running in the following active projects: - antigravity-manager-d58c5517 ..."`
- **Actual State:** Antigravity project `antigravity-manager-d58c5517` was actively executing instructions in conversation `d58c5517-d8ad-437e-ab7c-e506b0322383` with status `'CASCADE_RUN_STATUS_RUNNING'` and `not_fully_idle = 1`.

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Why did the system declare the workspace idle?
In `src-tauri/src/modules/email_watcher.rs` (lines 399-405):
```rust
let active_prompts = match crate::modules::repo_db::list_backed_up_prompts() {
    Ok(p) => p,
    Err(_) => return,
};
let has_prompts = !active_prompts.is_empty();
if !has_prompts {
    // SENDS IDLE NOTIFICATION
}
```
The sensor evaluated `list_backed_up_prompts()`, which runs `SELECT ... FROM active_prompts WHERE status = 'backed_up'`.

### 2.2 Why was `list_backed_up_prompts()` empty?
Because `active_prompts` only contains rows when the user or an automated script explicitly ran `agm backup-running-prompts`. In standard active development without manual backups, `status = 'backed_up'` has zero rows. The sensor equated "no backup records exist" with "the workspace has no active prompts and is idle".

### 2.3 Why was live Antigravity execution ignored?
Antigravity stores live conversation telemetry in `~/.gemini/antigravity/conversation_summaries.db`:
- `status`: `'CASCADE_RUN_STATUS_RUNNING'` vs `'CASCADE_RUN_STATUS_IDLE'`
- `not_fully_idle`: `1` (actively generating / running tool) vs `0` (idle)
- `last_modified_time`
The idle sensor in `email_watcher.rs` never checked `conversation_summaries.db` or `discover_running_prompts_from_antigravity()`, causing a 100% false-positive idle trigger whenever backups weren't manually executed.

### 2.4 Prevention & Code Fix
1. Inspect live Antigravity execution state:
   - Query `conversation_summaries.db` for conversations with `status == 'CASCADE_RUN_STATUS_RUNNING'` OR `not_fully_idle != 0`.
   - Match conversation workspace URIs to project paths.
2. If any monitored project is actively running, mark that project as `is_running = true` and `is_idle = false`.
3. If all projects are running or if any project is running, do not send false-positive notifications claiming "no active prompts are currently running". Only alert if all projects are genuinely idle.

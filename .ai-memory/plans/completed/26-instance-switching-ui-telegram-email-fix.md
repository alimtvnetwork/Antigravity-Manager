# Consolidated Completed Plan: Multi-Instance Switching Integrity, UI Modal Exclusivity, Conversation Pruner Safety Gate, and Telemetry 200w Preview

Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)
Status: `COMPLETED`
Execution Budget: N = 200 (Completed across 8 iterative self-loops)

## Summary of Accomplishments

1. **Multi-Instance Switching Bug & Persistence**:
   - Resolved instance bounce-back bug where selecting a non-default profile (#2, #3, #4) reverted to `worker-alpha` during periodic 4-second polling.
   - Updated `src-tauri/src/modules/instance.rs` (`get_active_instance_id()`) to prioritize `registry.active_instance_id` directly if present in `registry.instances`.
   - Updated `set_active_instance_id()` to stamp `last_used` timestamp.
   - Set Windows process spawn flag to `CREATE_NEW_PROCESS_GROUP` (`0x00000200`) in `launch_instance_process`.

2. **Focus, Scroll Position & High-Contrast Highlight Styling**:
   - Updated `src/components/navbar/InstanceSelector.tsx` to automatically scroll active instance into view using `scrollIntoView({ behavior: 'smooth', block: 'nearest' })` upon opening.
   - Replaced dark-on-dark invisible selection styling with high-contrast bright background, dark bold text (`text-slate-950 font-bold`), solid blue accent border (`border-l-blue-600`), and blue ring highlight.
   - Added success toast feedback on instance selection.
   - Updated `src/components/accounts/AccountRow.tsx` dark mode highlighting to vibrant high-contrast styles.

3. **UI Modal Exclusivity & Header Close Buttons**:
   - Added explicit close (`X`) buttons to both `InstanceSelector.tsx` header and `AgyCleanModal.tsx`.
   - Implemented `Escape` key dismissal and backdrop click dismissal.
   - Dispatched mutual exclusivity event (`agm:dropdown-open`) across `InstanceSelector`, `AgyCleanModal`, `NavMenu`, and `NavSettings` to ensure only one modal/dropdown is displayed at a time.

4. **Conversation Cleaner / Prune Safety Gate**:
   - Updated `src-tauri/src/modules/agy_cleaner.rs` (`scan_conversations()`):
     - Safely protected all running, queued, dispatched, and backed-up prompt session IDs from `repo_db::list_all_prompts()` and `.antigravity_resume_task.json`.
     - Automatically preserved at least the latest 5 conversation session files (`.vscdb`) for each active project workspace in `repo_db::list_running_projects()` before applying global `keep_count`.
   - Exposed `agm prune` and `agm clean-conversations` CLI commands in `src-tauri/src/bin/agm.rs` mapped to `cmd_clear_cache`.
   - Added `/prune` and `/clean` slash command handlers to Telegram bot daemon in `src-tauri/src/modules/telegram_inbound.rs`.
   - Added prominent active workspace and running prompt protection guarantee notice in `AgyCleanModal.tsx`.

5. **Email & Telegram Project Name Deduplication**:
   - Implemented `deduplicate_names` in `src-tauri/src/modules/notification_hub.rs`.
   - Deduplicated active project lists in switch emails, JSON self-broadcasts, Telegram switch alerts, and Telegram `/restore`, `/backup`, `/ff` handlers.

6. **Telegram Bot HTML Repair & AGM Update Command**:
   - Implemented `strip_telegram_html_tags` in `src-tauri/src/modules/telegram_inbound.rs` to strip tags and decode HTML entities before retrying plain text transmission on 400 Bad Request.
   - Escaped all dynamic fields in Telegram switch notifications with `escape_telegram_html`.
   - Added `/update` and `/upgrade` bot command handlers.
   - Added AGM update status indicator to `/status` (`format_observe_report`).
   - Expanded `/help` manual with prune commands and GitMap SSH fleet examples.

7. **Email 200-Word Running Prompts Preview & CLI Parity**:
   - Extracted up to 200 words from active prompt text using `extract_words_preview` in `notification_hub.rs`.
   - Rendered active prompt previews in email notifications and Telegram alerts.
   - Ensured `agm prompts ls --words 200` formats output cleanly with 200-word previews.

8. **End-to-End Verification & Evidence Generation**:
   - Validated end-to-end instance creation, real-time 5s heartbeat runner, PID verification, screenshot audit cards with exact timestamps, account switching, prompt backup in DB, instance reopening, and prompt re-injection.

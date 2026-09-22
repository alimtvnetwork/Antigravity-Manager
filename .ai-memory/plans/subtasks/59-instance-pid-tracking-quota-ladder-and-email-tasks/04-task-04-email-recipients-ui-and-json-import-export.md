# Subtask 04: Email Recipients UI Enhancement, Task Dispatch Dropdown & Universal JSON Settings Import/Export

Traceability ID: Task-04
Target Files:
- src/components/settings/EmailNotificationSettings.tsx
- src/pages/Email.tsx
- src-tauri/src/modules/email_sender.rs
- src-tauri/src/modules/email_io.rs
- src-tauri/src/commands/email.rs

Action:
- In `EmailNotificationSettings.tsx`, enhance the `[Send] Notification Recipients` card (from screenshot):
  - Add a "Test Task Dispatch" action area with a dropdown menu allowing selection of system task templates:
    - Prompt Injection (`prompt:`)
    - PowerShell Run (`powershell:`)
    - Command Run (`cmd:`)
    - GitMap Run (`gitmap:`)
    - Health / Status Check (`status:`)
  - Add an action button "Dispatch Test Email" to send the chosen task to registered recipients.
  - Add configurable check intervals in settings:
    - Periodic test interval (e.g. 5–10 minutes) until first incoming confirmation email is received.
    - Post-dispatch fast-poll interval (5–10 seconds).
- Implement universal JSON import and export for all email configurations, notification settings, registered recipients, and task schedules via `email_io.rs` and `commands/email.rs`.

Acceptance Criteria:
- Developer can select task types from dropdown and trigger test dispatch directly from UI.
- Fast-poll (5–10s) and periodic check (5–10m) intervals are configurable in UI and persisted.
- Full email configuration and recipient list can be exported to JSON and restored cleanly via import.

Targeted Verification:
- python 03-ai-scripts/05-guideline-autofixer.py src-tauri/src/modules/email_io.rs

Status: COMPLETED
- Implemented multi-pass base64 protection (`base64_encode_multi`, `base64_decode_multi`) for secrets in universal JSON export/import.
- Added `dispatch_custom_email_task` backend IPC command in `commands/email.rs` supporting `prompt`, `powershell`, `cmd`, `gitmap`, and `status` tasks.
- Enhanced `Notification Recipients` card in `EmailNotificationSettings.tsx` with Developer Task Quick Dispatch section, recipient selector, custom subject, and dispatch button.
- Added adaptive interval sliders (5–10 min initial wait, 5–10 sec fast-poll) and dedicated "Save Monitoring Intervals" button.

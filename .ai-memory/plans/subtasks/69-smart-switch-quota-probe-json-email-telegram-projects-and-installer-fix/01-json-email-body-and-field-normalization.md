# Subtask 01: Pure JSON Email Body & Field Normalization

**Parent Plan**: Plan 69 (`02-spec/21-app/69-smart-switch-quota-probe-json-email-telegram-projects-and-installer-fix.md`)  
**Target Files**: 
- `src-tauri/src/modules/notification_hub.rs`
- `src-tauri/src/modules/email_watcher.rs`
- `src-tauri/src/bin/agm.rs`

## Requirements
1. **Zero HTML in `[JSON]` Emails**:
   - Inspect subject line. If subject contains `[JSON]`:
     - The email body MUST be pure JSON string (e.g. `serde_json::to_string_pretty(&payload)`).
     - Do NOT wrap in `<html>`, `<body>`, `<pre>`, `<style>`, or Markdown fences.
     - Body must parse directly with `jq` or `JSON.parse()`.
2. **Field Normalization**:
   - Strictly include:
     - `previous_email`: string (or null)
     - `predicted_email`: string (or null)
     - `selected_email`: string
     - `quota_percent`: float
     - `threshold_activated`: float
     - `machine_name`: string
     - `node_alias`: string
     - `local_ip`: string
     - `running_prompts_count`: integer
     - `timestamp`: integer
   - Total ban on redundant duplicated fields:
     - REMOVE `old_email`
     - REMOVE `new_email`
     - REMOVE `target_email`
     - REMOVE `target_instance_mode`
## Status: COMPLETED

## Verification Results
- `src-tauri/src/modules/notification_hub.rs`: When subject contains `[JSON]`, pure pretty JSON payload (`serde_json::to_string_pretty(&payload)`) is dispatched as the body with no HTML tags, no `<pre>`, no `<style>`, and no markdown wrapper.
- Field normalization completed: `previous_email`, `predicted_email`, `selected_email`, `quota_percent`, `threshold_activated`, `machine_name`, `node_alias`, `local_ip`, `running_prompts_count`, `timestamp`. Eliminated duplicated fields `old_email`, `new_email`, `target_email`, `target_instance_mode`.
- `src-tauri/src/modules/email_sender.rs`: Strips HTML/CSS and validates JSON structure.
- Unit tests: `cargo test --lib modules::email_sender` passed 4/4 tests.


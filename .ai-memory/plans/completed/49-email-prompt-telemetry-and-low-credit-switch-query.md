# 49: Email Prompt Telemetry, Reinjection State & Low-Credit Switch CLI Queries

**Status:** Completed  
**Date:** 2026-09-26  
**Spec Reference:** [`02-spec/21-app/47-email-prompt-telemetry-and-low-credit-switch-query.md`](../../../02-spec/21-app/47-email-prompt-telemetry-and-low-credit-switch-query.md)

## Summary of Accomplishments
1. **Email Subject & From-To Account Integrity:**
   - Fixed the switch email alert subject in `src-tauri/src/modules/notification_hub.rs` to format `Account Switched: {from_display} -> {to_email}` (e.g. `Account Switched: previous@example.com -> alex.hudson.riseup@gmail.com` or `(none)`), eliminating the faulty instance name placeholder (`default`).
   - Updated `dispatch_telegram_switch_alert` to display `👤 Account: {from_display} ➔ {account_email}`.

2. **Running Prompt, Reinjection & Image Payload Telemetry:**
   - Queried `repo_db::list_all_prompts()` and fallbacks on `.antigravity_resume_task.json` across running projects to detect active tasks during switch events.
   - Extracted `running_prompt_id`, `running_prompt_snippet` (bounded to 120 chars), `running_prompt_project`, `is_reinjecting` (true upon snapshot generation), `has_images` (true if `image_payload` is present), and `images_attached`.
   - Rendered dedicated rows in the HTML table (`Running Prompt`, `Re-injecting Task`, `Attached Images`) and rich fields in the `[JSON]` state machine block.

3. **CLI Low-Credit Switch File Export (`swlc -f`):**
   - Added `-f [path]` / `--file [path]` support to `agm switch-if-low-credit` (`swlc` / `sfc`).
   - Defaulted filename to `agm-<node_alias>-switch.json` (e.g. `agm-VM3-switch.json`) with sanitize fallback to `agm-node-switch.json`.
   - Writes full switch evaluation JSON payload to disk.

4. **CLI Low-Credit Switch Query (`agm is-low-credit-for-switch` / `ilc`):**
   - Implemented `cmd_is_low_credit_for_switch` without mutating account states or triggering rotation.
   - Plain text mode: outputs strictly `true` or `false` to stdout and exits with code 0.
   - JSON mode (`--json`): emits machine identity (`machine_name`, `node_alias`, `local_ip`), current account quota status, threshold, target model, tool version, next candidate standby account and quota, and active prompt/image telemetry.
   - File export mode (`-f [path]`): writes JSON payload to destination file or `agm-<node_alias>-switch.json` by default.
   - Registered `is-low-credit-for-switch`, `is-low-credit`, and `ilc` in `main()` command dispatcher and updated `agm --help`.

## Subtasks Completed
- [x] `01-email-prompt-image-telemetry.md` — Enrich email switch alerts with From-To account integrity (`from_email -> to_email`), running prompt snippet, reinjection status (`is_reinjecting`), and image payload attachment status in both HTML table and `[JSON]` block.
- [x] `02-cli-low-credit-query-and-file-export.md` — Support `-f [path]` file export with `agm-<node_alias>-switch.json` default in `agm switch-if-low-credit` (`swlc`), and implement `agm is-low-credit-for-switch` (`ilc`) returning boolean in plain mode and comprehensive JSON in `--json [-f]` mode.

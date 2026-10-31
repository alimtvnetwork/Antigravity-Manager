# Subtask 02: Idle Mode Auto-Notification Hub for Telegram and Email

## Status
- **Status**: Completed
- **Completed At**: 2026-10-28T18:46:00+08:00

## Target Files
- `src-tauri/src/modules/notification_hub/idle_alerts.rs`
- `src-tauri/src/modules/notification_hub/mod.rs`
- `src-tauri/src/modules/email_watcher/sensors.rs`

## Objectives & Outcomes
1. **Implemented `src-tauri/src/modules/notification_hub/idle_alerts.rs`**:
   - `dispatch_idle_mode_alert(instance_id: &str, instance_name: &str, idle_duration_secs: u64) -> Result<String, String>` coordinates unified notifications across Telegram and Email channels.
   - Dispatches rich notification to Telegram using `telegram_inbound::send_telegram_message_with_markup` when Telegram bot is configured and enabled.
   - Formats message with machine name, IP address, instance identifier, idle duration seconds, prompt command hint, and app version.
   - Includes interactive inline button markup (`/send` prompt action and copy helper).
   - Dispatches HTML email using `email_sender::render_idle_projects_email` and `email_sender::dispatch_email_with_failover`.
   - Enforces 600-second (10-minute) deduplication cooldown per `instance_id` with `LAST_IDLE_ALERT_PER_INSTANCE` Mutex map.
2. **Exported in `src-tauri/src/modules/notification_hub/mod.rs`**:
   - Registered `pub mod idle_alerts;` and `pub use idle_alerts::*;`.
3. **Integrated into `src-tauri/src/modules/email_watcher/sensors.rs`**:
   - Updated `check_idle_projects_sensor` to resolve target instance name/ID and idle duration, delegating multi-channel dispatch to `crate::modules::notification_hub::dispatch_idle_mode_alert`.
   - Ensured robust error tracking using `crate::error::record_ignored` with standard justification comment.

## Verification
- Unit tests in `modules::notification_hub::idle_alerts::tests`:
  - `test_idle_alert_formatting_components` (PASSED)
  - `test_idle_alert_cooldown` (PASSED)
  - `test_idle_alert_inline_keyboard` (PASSED)
- `cargo fmt -- --check`: Clean pass with 0 formatting discrepancies.
- `cargo clippy --lib`: Clean pass with 0 errors.

# Subtask 02: Telegram Alert HTML Fix, Rich Keyboards, `/balance` & CSV/JSON File Export

> **Subtask Path:** `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/02-telegram-bot-and-csv-export.md`  
> **Architecture Spec:** `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md`  

---

## 1. Objective

Resolve the literal `&nbsp;&nbsp;` HTML rendering defect in Telegram notifications, add rich inline keyboard action buttons to account rotation and liveness alerts, implement the `/balance` (`/credits`, `/quota`) command and `/export` CSV/JSON multipart document sender (`sendDocument`), streamline machine/instance prompt targeting, and upgrade the `/help` manual.

---

## 2. Owned Target Files (Strictly Disjoint)

1. `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
2. `src-tauri/src/modules/notification_hub/telemetry.rs`
3. `src-tauri/src/modules/telegram_inbound/formatting.rs`
4. `src-tauri/src/modules/telegram_inbound/media.rs`
5. `src-tauri/src/modules/telegram_inbound/reports_a.rs`
6. `src-tauri/src/modules/telegram_inbound/reports_b.rs`
7. `src-tauri/src/modules/telegram_inbound/processing.rs`
8. `src-tauri/src/modules/telegram_inbound/daemon.rs`
9. `src-tauri/src/modules/telegram_inbound/send_flow.rs`
10. `src-tauri/src/modules/telegram_inbound/tests.rs`

---

## 3. Detailed Implementation Requirements

### 3.1 Fix Literal `&nbsp;&nbsp;` HTML Entity Bug & Add Rich Inline Buttons on Alerts

- **`src-tauri/src/modules/notification_hub/telegram_alerts.rs`:**
  1. Replace `&nbsp;&nbsp;└ <i>Balances:</i>` (lines 191 and 193) with `\u{2007}\u{2007}└ <i>Balances:</i>` so Telegram's HTML parser renders clean figure-space indentation instead of literal `&nbsp;&nbsp;`.
  2. Replace `links.join(" &bull; ")` (line 159) with `links.join(" • ")`.
  3. Always attach a rich inline keyboard to switch alerts via `telegram_inbound::build_switch_alert_reply_markup(&details.instance_name, &details.selected_email, &target_4h_str, &target_weekly_str, &full_prompt_content, &resolved_image_urls)` instead of returning `None` when `full_prompt_content` is empty.
- **`src-tauri/src/modules/notification_hub/telemetry.rs`:**
  1. Replace `links.join(" &bull; ")` (line 104) with `links.join(" • ")`.
  2. Attach a rich inline keyboard to post-switch liveness and emergency alerts (`telegram_inbound::build_liveness_alert_reply_markup(instance_id, verified_running, backed_up_count, &full_prompt_content, &detected_images)`) with buttons:
     - Row 1: `🟢 {instance_id} | Running: {verified_running}/{backed_up_count}` (`callback_data: "cmd:active"`)
     - Row 2: `💰 Check Balance` (`callback_data: "cmd:balance"`), `📥 Export CSV` (`callback_data: "cmd:export balance csv"`)
     - Row 3: `💬 Prompt Instance` (`callback_data: "cmd:send"`), `🔄 Refresh Status` (`callback_data: "cmd:observe"`)
     - Plus `📋 Copy Prompt` if `full_prompt_content` is non-empty.
- **`src-tauri/src/modules/telegram_inbound/formatting.rs`:**
  1. Add `pub fn sanitize_telegram_html_entities(text: &str) -> String` that replaces `&nbsp;` with `\u{2007}` and `&bull;` with `•`, and invoke it inside `send_telegram_message_with_markup` before chunking.
  2. Implement `build_switch_alert_reply_markup` and `build_liveness_alert_reply_markup`.

### 3.2 Dedicated `/balance` (`/credits`, `/quota`) Command

- **`src-tauri/src/modules/telegram_inbound/reports_a.rs`:**
  1. Implement `pub fn format_balance_report(filter: &str) -> String`:
     - Displays machine identity (`hostname`, `ip`, `alias`), quota protection threshold (`threshold_percentage`), and total accounts count.
     - Lists every registered instance (`instance::load_registry()`), its live process state (`is_instance_process_running_smart`), bound account email, subscription tier, 4h quota %, weekly quota %, and model breakdown.
     - Displays the top standby accounts with highest 4h & weekly balances ready for rotation.
  2. Implement `pub fn resolve_balance_markup() -> serde_json::Value` with buttons:
     - `[🔄 Refresh Balance (cmd:balance)]`, `[⚡ Fast-Forward (cmd:ff)]`
     - `[📥 Export CSV (cmd:export balance csv)]`, `[📄 Export JSON (cmd:export balance json)]`
     - `[🏷️ Instances (cmd:instances)]`, `[💬 Prompt Instance (cmd:send)]`
- **`src-tauri/src/modules/telegram_inbound/processing.rs` & `daemon.rs`:**
  - Wire `"balance" | "credits" | "quota" | "bal"` to `format_balance_report(rest)` and attach `resolve_balance_markup()`.

### 3.3 Multipart File Upload (`sendDocument`) & CSV/JSON Exports (`/export`, `--csv`, `--json`)

- **`src-tauri/src/modules/telegram_inbound/media.rs`:**
  1. Implement `pub async fn send_telegram_document(bot_token: &str, chat_id: i64, file_bytes: &[u8], file_name: &str, mime_type: &str, caption: &str, reply_markup: Option<serde_json::Value>) -> Result<String, crate::error::AppError>` using `reqwest::multipart::Form` and `/bot{token}/sendDocument`.
- **`src-tauri/src/modules/telegram_inbound/reports_a.rs`:**
  1. Implement `pub struct TelegramExportDocument { pub file_name: String, pub mime_type: String, pub bytes: Vec<u8>, pub caption: String, pub markup: Option<serde_json::Value> }`.
  2. Implement `pub fn generate_export_document(kind: &str, format: &str) -> Result<TelegramExportDocument, String>` supporting:
     - `balance` / `quota` / `credits`: columns `machine,ip,instance_id,instance_name,running,account_email,tier,quota_4h_pct,quota_weekly_pct,min_model_pct,threshold_pct`
     - `accounts`: columns `id,email,tier,is_active,bound_instances,quota_4h_pct,quota_weekly_pct,min_model_pct`
     - `instances`: columns `id,name,is_default,running,pid,bound_email,data_dir`
     - `nodes` / `ssh`: columns `alias,ip_address,username,port,os,auth_method`
     - `prompts`: columns `id,instance_id,project_id,status,word_count,created_at,preview`
- **`src-tauri/src/modules/telegram_inbound/daemon.rs`:**
  - Intercept `/export <kind> [csv|json]` as well as `--csv` / `--json` flags on `/balance`, `/accounts`, `/instances`, `/nodes`, and `/prompts`.
  - When triggered in `deliver_ui_or_command` or callback query `cmd:export ...`, upload the generated `.csv` or `.json` file via `send_telegram_document` and also return a formatted summary caption.

### 3.4 Streamlined Machine & Instance Prompting & Rich `/help` Manual

- **`src-tauri/src/modules/telegram_inbound/processing.rs` & `send_flow.rs`:**
  - Enhance `/prompt` so `/prompt <instance_or_node> <prompt_text>` and `/prompt --node <node> --instance <inst> <text>` route directly to the specified local instance or remote SSH node when the target is not a `C001`/`P001` sequence ID.
  - Enhance `/target` so `/target <machine> <instance> [conversation]` sets the target aim across machine/instance/conversation.
- **`src-tauri/src/modules/telegram_inbound/reports_b.rs`:**
  - Update `format_help_manual()` to document `/balance`, `/credits`, `/quota`, `/export <balance|accounts|instances|nodes|prompts> [csv|json]`, `--csv`/`--json` flags, `/prompt <machine|instance> <text>`, and `/ssh import-gitmap` / `/ssh balance` with click-to-copy `<code>` blocks.

---

## 4. Acceptance Criteria

- [ ] Zero occurrences of `&nbsp;` or `&bull;` remain in `src-tauri/src/modules/notification_hub/telegram_alerts.rs` or `src-tauri/src/modules/notification_hub/telemetry.rs`.
- [ ] Switch and post-switch liveness alerts always include rich inline keyboard buttons (`Check Balance`, `Export CSV`, `Prompt Instance`, `Refresh Status`).
- [ ] `/balance` (`/credits`, `/quota`) returns a rich HTML card with per-instance active account, 4h & weekly quota percentages, and inline buttons.
- [ ] `/export <balance|accounts|instances|nodes|prompts> [csv|json]` and `--csv`/`--json` flags upload a downloadable `.csv` or `.json` document via `send_telegram_document`.
- [ ] Unit tests in `src-tauri/src/modules/telegram_inbound/tests.rs` pass and verify HTML entity sanitization, `/balance` output, CSV/JSON document generation, and `/help` formatting.

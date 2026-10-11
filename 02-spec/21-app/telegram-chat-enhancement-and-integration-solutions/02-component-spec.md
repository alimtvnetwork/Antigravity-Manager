# Component Specification: Telegram Chat Enhancement, Free Image Storage & Supabase-Free Fleet Discovery

## 1. Executive Summary & Design References

This component specification formalizes the architecture, data schemas, module interfaces, and lifecycle algorithms for three interconnected capabilities within Antigravity-Manager (AGM):
1. **Interactive Telegram Rich HTML Chat Cards & One-Tap Prompt Copying**: Redesigning the notification and inbound chat presentation to replace plain text dumps with high-contrast, structured Telegram HTML cards, collapsible viewport quotes (`<blockquote expandable>`), and Telegram Bot API 7.8+ inline clipboard copy buttons (`copy_text`).
2. **Zero-Cost Multi-Tier Image Storage Pipeline & Reusable Assets**: Providing permanent, 100% free image uploading, disk caching, and SQLite indexing using Telegram Bot Cloud storage (`sendPhoto` -> permanent `file_id`), Catbox.moe public hosting, and Litterbox ephemeral fallback. Assets are stored using Dual-Mode compound URIs (`tg:<file_id>|catbox:<public_url>|local:<local_path>`).
3. **Supabase-Free Fleet Identity Discovery**: Providing complete multi-machine, multi-instance, and bound-account topology visibility over both Telegram bot commands (`/fleet`, `/whoami`, `/instances`, `/status`, `/accounts`) and inbound Email commands (`FLEET`, `STATUS`), unified by a standardized Telemetry Header.

### 1.1 Visual Design Reference & Gap Analysis
As captured in the user's reference screenshot (`assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png`), the legacy Telegram notification card exhibits several critical UX limitations:
- **Unformatted Prompt Dump**: Prompts appear as raw unescaped text directly in the message body, stretching chat history and truncating awkwardly on mobile viewports.
- **Missing Interaction & Copy Actions**: Users cannot easily copy the active prompt or attached media markdown from the card to paste into external AI tools or CLI windows.
- **Lack of Asset Hyperlinks**: When prompts contain attached diagrams, screenshots, or code artifacts, there are no preview links or hosted URLs.
- **Limited Fleet Context**: Machine identity is restricted to a simple `Host: W3 (192.168.1.20)` line without showing running instances, bound accounts, memory utilization, or multi-node cluster topology when operating without Supabase sync.

```
+----------------------------------------------------------------------------------------------------+
| LEGACY NOTIFICATION (assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png) |
| Plain text, no copy button, massive inline prompt dump, minimal node context                      |
+----------------------------------------------------------------------------------------------------+
                                                |
                                                v
+----------------------------------------------------------------------------------------------------+
| ENHANCED RICH CARD SPECIFICATION                                                                  |
| • Telegram HTML <b> / <code> / <blockquote> styling                                                |
| • <blockquote expandable> collapsible prompt preview viewport                                      |
| • Telegram Bot API 7.8 inline copy buttons: [📋 Copy Prompt] & [🖼️ Copy Prompt + Images]           |
| • Standardized Telemetry Header: 🖥️ Machine | 🏷️ Instance | 👤 Bound Account                       |
| • Permanent Free Image URLs & Dual-Mode URIs (tg:<id>|catbox:<url>|local:<path>)                    |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Component Specification: Free Image Storage (`src-tauri/src/modules/telegram_inbound/media.rs`)

### 2.1 Subsystem Responsibilities
The `media.rs` module delivers a resilient, multi-tiered, zero-cost media lifecycle engine. It manages media ingestion from incoming Telegram messages, media persistence to local app storage, automated upload to free public CDNs, and persistent tracking in SQLite.

```
                      +-----------------------------+
                      | Incoming Telegram Photo /   |
                      | Local Prompt Media Path     |
                      +-----------------------------+
                                     |
                                     v
                      +-----------------------------+
                      | Local Disk Cache            |
                      | (~/.antigravity_tools/      |
                      |  saved_images/<timestamp>)  |
                      +-----------------------------+
                                     |
                                     v
                      +-----------------------------+
                      | Compute SHA-256 Hash        |
                      +-----------------------------+
                                     |
                +--------------------+--------------------+
                |                                         |
                v (Cache Hit in media_vault)              v (Cache Miss)
    +-----------------------+               +-------------------------------+
    | Instant URI Reuse     |               | Tier 1: Catbox.moe Upload     |
    | Return Compound URI   |               | POST https://catbox.moe/...   |
    +-----------------------+               +-------------------------------+
                                                          |
                                           +--------------+--------------+
                                           | (Success)                   | (Failure / Timeout)
                                           v                             v
                              +--------------------------+  +-------------------------------+
                              | Permanent Public URL     |  | Tier 2: Litterbox Ephemeral   |
                              | https://files.catbox.moe |  | POST https://litterbox.catbox |
                              +--------------------------+  +-------------------------------+
                                           |                             |
                                           +--------------+--------------+
                                                          |
                                                          v
                                            +-------------------------------+
                                            | Tier 3: Telegram sendPhoto    |
                                            | Captures permanent file_id    |
                                            +-------------------------------+
                                                          |
                                                          v
                                            +-------------------------------+
                                            | Store in media_vault (SQLite) |
                                            | Return CompoundMediaUri       |
                                            +-------------------------------+
```

### 2.2 Function Interfaces

#### 1. `send_telegram_photo`
Transmits an image binary directly to the Telegram chat, leveraging Telegram's cloud storage infrastructure to obtain a permanent `file_id`.
```rust
pub async fn send_telegram_photo(
    bot_token: &str,
    chat_id: i64,
    img_bytes: &[u8],
    caption: Option<&str>,
) -> Result<String, String>
```
- **Inputs**:
  - `bot_token`: Telegram Bot authentication token.
  - `chat_id`: Destination Telegram chat ID.
  - `img_bytes`: Raw image bytes (`PNG`, `JPEG`, `WEBP`).
  - `caption`: Optional HTML-formatted photo caption.
- **Outputs**: `Result<String, String>` where `Ok(file_id)` returns Telegram's permanent `file_id` from `resp["result"]["photo"].last()["file_id"]`.
- **Implementation Rules**:
  - Uses `multipart/form-data` with `chat_id`, `photo` part (named `image.png`), and `caption`.
  - Sets request timeout to 30 seconds.
  - On error, routes through `redact_secrets(&e.to_string())`.

#### 2. `upload_image_to_catbox`
Uploads raw image bytes to Catbox.moe's free anonymous public API for permanent web hosting.
```rust
pub async fn upload_image_to_catbox(
    img_bytes: &[u8],
    filename: &str,
) -> Result<String, String>
```
- **Endpoint**: `POST https://catbox.moe/user/api.php`
- **Multipart Form Payload**:
  - `reqtype`: `"fileupload"`
  - `fileToUpload`: Binary part with given `filename` and MIME type `image/png`.
- **Outputs**: `Result<String, String>` returning direct HTTPS URL (e.g., `https://files.catbox.moe/a1b2c3.png`).
- **Invariants**:
  - Zero API key requirement (anonymous public tier).
  - Permanent file retention.
  - Verifies response body starts with `"http"`. Any text response not starting with `"http"` is treated as an API rejection error.

#### 3. `upload_image_to_litterbox`
Ephemeral fallback upload when Catbox primary API returns a 5xx error or times out.
```rust
pub async fn upload_image_to_litterbox(
    img_bytes: &[u8],
    filename: &str,
    time_hours: u32,
) -> Result<String, String>
```
- **Endpoint**: `POST https://litterbox.catbox.moe/resources/internals/api.php`
- **Multipart Form Payload**:
  - `reqtype`: `"fileupload"`
  - `time`: Allowed retention values `"1h"`, `"12h"`, `"24h"`, `"72h"`. Defaults to `"72h"`.
  - `fileToUpload`: Binary part.
- **Outputs**: `Result<String, String>` returning ephemeral HTTPS URL (`https://litterbox.catbox.moe/...`).

#### 4. `upload_local_image_file`
High-level file upload wrapper with SQLite caching and dual-tier fallback.
```rust
pub async fn upload_local_image_file(path: &std::path::Path) -> Result<String, String>
```
- Computes SHA-256 hash of the target file.
- Checks `media_vault` SQLite table for existing `public_url`. If found, returns immediately.
- If not found, reads file bytes and executes `upload_image_to_catbox`. If Catbox fails, falls back to `upload_image_to_litterbox`.
- On success, inserts a new record into `media_vault`.

### 2.3 Dual-Mode Compound URI Specification
To ensure seamless asset portability across local offline nodes, the Telegram bot interface, and public web markdown viewers, media references utilize a compound URI format:

```text
tg:<file_id>|catbox:<public_url>|local:<local_path>
```

#### Compound Structure & Parsing:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundMediaUri {
    pub telegram_file_id: Option<String>,
    pub public_url: Option<String>,
    pub local_path: Option<String>,
}

impl CompoundMediaUri {
    pub fn parse(raw: &str) -> Self {
        let mut uri = Self::default();
        for segment in raw.split('|') {
            let seg = segment.trim();
            if let Some(val) = seg.strip_prefix("tg:") {
                uri.telegram_file_id = Some(val.to_string());
            } else if let Some(val) = seg.strip_prefix("catbox:") {
                uri.public_url = Some(val.to_string());
            } else if let Some(val) = seg.strip_prefix("local:") {
                uri.local_path = Some(val.to_string());
            } else if seg.starts_with("http://") || seg.starts_with("https://") {
                uri.public_url = Some(seg.to_string());
            }
        }
        uri
    }

    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref tg) = self.telegram_file_id {
            parts.push(format!("tg:{}", tg));
        }
        if let Some(ref url) = self.public_url {
            parts.push(format!("catbox:{}", url));
        }
        if let Some(ref loc) = self.local_path {
            parts.push(format!("local:{}", loc));
        }
        parts.join("|")
    }

    pub fn best_markdown_url(&self) -> Option<&str> {
        self.public_url.as_deref().or(self.local_path.as_deref())
    }
}
```

### 2.4 SQLite Schema: `media_vault` Table
Stored in the existing split database `repo_prompts.db` to maintain zero-loss prompt continuity:

```sql
CREATE TABLE IF NOT EXISTS media_vault (
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE,
    local_path TEXT NOT NULL,
    public_url TEXT,
    telegram_file_id TEXT,
    mime_type TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_media_vault_sha256 ON media_vault (sha256);
```

---

## 3. Component Specification: Supabase-Free Fleet Identity & Telemetry

### 3.1 Subsystem Responsibilities
When Supabase PostgreSQL clustering is unconfigured or offline, multi-machine operators running AGM across local desktop rigs, remote workstations, headless VM runners, and laptops need total clarity regarding:
- Which physical host and IP address executed or triggered an operation.
- Which named instance profile and data directory were modified.
- Which specific account email was selected and bound.
- Real-time resource status, running PIDs, and active prompts.

### 3.2 Standardized Telemetry Header
Every outbound Telegram notification, inbound Telegram command report, outbound Email alert, and Email command receipt MUST prepend or include the canonical Telemetry Header:

```text
🖥️ Machine: <hostname> (<ip>) | 🏷️ Instance: <instance_id> | 👤 Bound: <email>
```

#### Format Variations:
1. **Telegram HTML Header**:
   ```html
   🖥️ <b>Machine:</b> <code>W3 (192.168.1.20)</code> | 🏷️ <b>Instance:</b> <code>default</code> | 👤 <b>Bound:</b> <code>zihadxshohag2@gmail.com</code>
   ```
2. **Email Subject Tuple Standard**:
   ```text
   [AGM v4.162.0 | W3 (192.168.1.20) | default | zihadxshohag2@gmail.com] Account Switched: rokixshohag1@gmail.com -> zihadxshohag2@gmail.com
   ```
3. **Plain Text Receipt Header**:
   ```text
   Machine:          W3 (192.168.1.20)
   Target Instance:  default (ID: default)
   Bound Account:    zihadxshohag2@gmail.com
   ```

### 3.3 Enhanced Telegram Reports (`src-tauri/src/modules/telegram_inbound/reports_a.rs`)

#### 1. Standalone Fleet Identity Card (`/fleet`, `/whoami`, `/machines`)
Enhanced `format_fleet_identity_report()` constructs a complete standalone node report:
- **Host & System Info**: Machine hostname (`COMPUTERNAME` / `HOSTNAME`), LAN IP (`detect_local_ip()`), OS architecture, application version (`git_info::get_app_version()`).
- **Active Instance State**: Active profile name, profile ID, data directory path, live OS PIDs (`find_pids_for_data_dir()`), and execution state (`🟢 RUNNING` vs `⚪ IDLE`).
- **Bound Account Context**: Active bound email address, subscription tier, and current dual-window quota balances (`4h` and `7-day weekly`).
- **Local Fleet Inventory**: Total registered sandbox instances and total registered accounts.
- **Actionable Hints**: Inline command suggestions for `/instances`, `/accounts`, `/status`, and `/observe`.

#### 2. Detailed Instance Profile Table (`/instances`)
Enhanced `format_instances_report()` iterates over `instance::load_registry()`:
- Numbered list of profiles with status emojis (`🟢` running / `⚪` idle).
- Monospace profile name, profile ID, active badge (`[ACTIVE]`).
- Bound account email and sandboxed data directory path.

#### 3. Dual-Window Quota Accounts Table (`/accounts`)
Enhanced `format_accounts_report()` iterates over `account::list_accounts()`:
- Monospace email and subscription tier (`PRO`, `FREE`, `ULTRA`).
- Accurate 4-hour rolling and 7-day weekly quota percentages extracted via `auto_switcher::extract_dual_window_quotas(acc, "gemini-2.5-pro")`.
- Active account indicator.

#### 4. Live Workspace Status (`/status`)
Enhanced `format_observe_report()` provides real-time running prompt snapshots:
- Workspaces with running tasks, formatted durations, and 200-word prompt previews.
- Inline `/expand <id>` shortcuts for instant inspection.

### 3.4 Email Inbound Command Parsing Extensions

#### 1. Legacy & Flexible Prefix Grammar (`src-tauri/src/modules/email_inbound/parse_legacy.rs`)
Updated to recognize `FLEET`, `WHOAMI`, `CLUSTER`, and standalone `STATUS`:
```rust
// In parse_legacy_prefix_command:
if lower_subj == "fleet"
    || lower_subj == "whoami"
    || lower_subj == "machines"
    || lower_subj == "agm fleet"
    || lower_subj == "agm whoami"
{
    return InboundAction::StatusQuery {
        target: "*".to_string(),
    };
}
```

#### 2. Unified Pipe Grammar (`src-tauri/src/modules/email_inbound/parse_pipe.rs`)
Updated to support pipe-delimited node targeting for fleet commands:
```rust
// In parse_pipe_delimited_command:
if lower_cmd == "fleet" || lower_cmd == "agm fleet" || lower_cmd == "whoami" {
    return Some(InboundAction::StatusQuery {
        target: target.to_string(),
    });
}
```

#### 3. Execution & Receipt Response (`src-tauri/src/modules/email_inbound/handlers_b.rs`)
`handle_status_query` is updated to include the standardized Telemetry Header and complete instance/account inventory in both ACK receipts and final execution outcome emails.

---

## 4. Component Specification: Telegram Notification Hub (`telegram_alerts.rs`)

### 4.1 HTML Visual Hierarchy
`src-tauri/src/modules/notification_hub/telegram_alerts.rs` formats `SwitchNotificationDetails` into a cohesive, high-contrast visual card:
1. **Header Banner**: `🚀 <b>ANTIGRAVITY MANAGER</b> | <b>ACCOUNT ROTATION</b> ⚡`
2. **Fleet Topology Card**: Machine name, LAN IP, target instance ID, active bound email.
3. **Quota Delta Card**: Previous account balances (`4h` & `Weekly`) vs Target account balances (`4h` & `Weekly`), and switch threshold percentage.
4. **Workspace Context**: Backed-up workspace project names, captured prompt count, and switch trigger reason.
5. **Expandable Prompt Quote**: Active prompt content wrapped inside:
   ```html
   <blockquote expandable><pre><code class="language-markdown">{clean_prompt_preview}</code></pre></blockquote>
   ```
6. **Attached Media Section**: When images are resolved via `media_vault` or uploaded to Catbox, clickable hyperlinks (`<a href="...">🖼️ Image #1</a>`) appear directly above the keyboard.

### 4.2 Inline Keyboard & Safe Copy Bounds
Telegram Bot API 7.8 introduces `copy_text` within `InlineKeyboardButton`:
```json
{
  "text": "📋 Copy Prompt",
  "copy_text": {
    "text": "<full_prompt_content>"
  }
}
```

#### Invariant: Safe Payload Bounding
Telegram Bot API strictly enforces a maximum payload limit of 4,096 characters on inline button parameters. Exceeding this limit causes an immediate HTTP 400 rejection (`BUTTON_DATA_INVALID`).
- AGM enforces a safe boundary of **3,900 UTF-8 characters**.
- When clamping strings, AGM verifies UTF-8 character boundaries (`is_char_boundary(end)`), stepping backwards to prevent slicing multi-byte unicode glyphs.
- When attached images exist, AGM generates a secondary button `[🖼️ Copy Prompt + Images]` combining the prompt text with markdown image tags:
  ```markdown
  {prompt_text}

  ---
  ### Attached Images:
  ![image](https://files.catbox.moe/abc123.png)
  ```

---

## 5. Summary of Module Changes & Deliverables

| Target Module | Function / Struct | Action | Invariant / Key Requirement |
| :--- | :--- | :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/media.rs` | `send_telegram_photo` | Implement | Sends photo via Telegram Bot API, capturing permanent `file_id`. |
| `src-tauri/src/modules/telegram_inbound/media.rs` | `upload_image_to_catbox` | Refactor / Enhance | Uploads to Catbox.moe free public CDN; returns `https://files.catbox.moe/...`. |
| `src-tauri/src/modules/telegram_inbound/media.rs` | `upload_image_to_litterbox` | Implement | Ephemeral 72h fallback upload to Litterbox. |
| `src-tauri/src/modules/telegram_inbound/media.rs` | `CompoundMediaUri` | Implement | Dual-Mode URI parser and formatter (`tg:<id>\|catbox:<url>\|local:<path>`). |
| `src-tauri/src/modules/telegram_inbound/reports_a.rs` | `format_fleet_identity_report` | Enhance | Telemetry Header, instance PIDs, bound accounts, dual-window quotas. |
| `src-tauri/src/modules/notification_hub/telegram_alerts.rs` | `dispatch_telegram_switch_alert` | Enhance | HTML cards, `<blockquote expandable>`, `copy_text` <= 3900 chars. |
| `src-tauri/src/modules/email_inbound/parse_legacy.rs` | `parse_legacy_prefix_command` | Enhance | Accepts `FLEET`, `WHOAMI`, `CLUSTER` for standalone machine queries. |
| `src-tauri/src/modules/email_inbound/parse_pipe.rs` | `parse_pipe_delimited_command` | Enhance | Pipe syntax `* \| fleet` and `* \| whoami`. |
| `src-tauri/src/modules/email_inbound/handlers_b.rs` | `handle_status_query` | Enhance | Telemetry Header formatted in ACK receipts and execution results. |

# AI Instructions: Telegram Chat Enhancement, Free Image Storage & Supabase-Free Fleet Discovery

## 1. Operating Rules & Core Invariants

All executing AI worker agents assigned to this feature must strictly adhere to the following non-negotiable standards:

1. **Strict Relative Git Paths**:
   - Every file path, code import, artifact reference, and log must use relative Git paths (e.g., `src-tauri/src/modules/telegram_inbound/media.rs`, `02-spec/...`, `.ai-memory/...`).
   - Zero absolute filesystem paths are permitted anywhere in code, comments, specs, or documentation.
2. **Positive Boolean Naming Invariant**:
   - All boolean variables, struct fields, and functions must use affirmative, positive nomenclature (e.g., `is_active`, `has_images`, `is_running`, `is_success`, `has_match`, `is_valid`).
   - Negative prefixes (e.g., `is_not_empty`, `is_unassigned`, `disable_feature`) are strictly forbidden.
3. **Search Invariant**:
   - Use GitMap commands (`gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap ps`, `gitmap py`).
   - Absolute ban on raw search tools (`rg`, `ripgrep`, `grep`, `git grep`, `Select-String`).
4. **No Silent Errors Invariant**:
   - Every fallible operation must route its error into `AppError` (`src-tauri/src/error.rs`) or log via `tracing`/`record_ignored`.
   - Never use bare `.ok()`, `let _ = ...`, or empty `catch {}` blocks without a `// Justification:` comment and `crate::error::record_ignored(...)`.
5. **Design Reference Alignment**:
   - Compare all formatting output against the reference screenshot in `assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png`. Ensure that flat unformatted text dumps are transformed into structured HTML cards with collapsible `<blockquote expandable>` sections and inline copy buttons.

---

## 2. File-by-File Code Transformation Recipes

### Recipe 1: Free Image Storage & Dual-Mode Media Pipeline
- **Target File**: `src-tauri/src/modules/telegram_inbound/media.rs`
- **Objective**: Implement multi-tier free image hosting, disk caching, Telegram cloud storage integration, and SQLite indexing.

#### Transformation Steps:
1. Define the `CompoundMediaUri` struct and parser:
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
2. Implement `send_telegram_photo`:
   ```rust
   /// Send photo to Telegram chat and capture permanent file_id from Telegram Cloud
   pub async fn send_telegram_photo(
       bot_token: &str,
       chat_id: i64,
       img_bytes: &[u8],
       caption: Option<&str>,
   ) -> Result<String, String> {
       let client = reqwest::Client::builder()
           .timeout(Duration::from_secs(30))
           .build()
           .map_err(|e| format!("HTTP client init failed: {}", e))?;

       let part = reqwest::multipart::Part::bytes(img_bytes.to_vec())
           .file_name("image.png")
           .mime_str("image/png")
           .map_err(|e| format!("Invalid multipart part: {}", e))?;

       let mut form = reqwest::multipart::Form::new()
           .text("chat_id", chat_id.to_string())
           .part("photo", part);

       if let Some(cap) = caption {
           form = form.text("caption", cap.to_string()).text("parse_mode", "HTML");
       }

       let send_photo_url = format!("https://api.telegram.org/bot{}/sendPhoto", bot_token);
       let resp = client
           .post(&send_photo_url)
           .multipart(form)
           .send()
           .await
           .map_err(|e| format!("sendPhoto request failed: {}", e))?;

       if !resp.status().is_success() {
           return Err(format!("Telegram sendPhoto returned HTTP {}", resp.status()));
       }

       let body: Value = resp.json().await.map_err(|e| e.to_string())?;
       let photos = body["result"]["photo"]
           .as_array()
           .ok_or_else(|| "Missing photo array in sendPhoto response".to_string())?;
       let best = photos.last().ok_or_else(|| "Empty photo array".to_string())?;
       let file_id = best["file_id"]
           .as_str()
           .ok_or_else(|| "Missing file_id".to_string())?;

       Ok(file_id.to_string())
   }
   ```
3. Implement `upload_image_to_litterbox` fallback:
   ```rust
   /// Upload to Litterbox (ephemeral 72h fallback if Catbox is unreachable)
   pub async fn upload_image_to_litterbox(
       img_bytes: &[u8],
       filename: &str,
       time_hours: u32,
   ) -> Result<String, String> {
       let client = reqwest::Client::builder()
           .timeout(Duration::from_secs(30))
           .build()
           .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

       let time_str = match time_hours {
           1 => "1h",
           12 => "12h",
           24 => "24h",
           _ => "72h",
       };

       let part = reqwest::multipart::Part::bytes(img_bytes.to_vec())
           .file_name(filename.to_string())
           .mime_str("image/png")
           .map_err(|e| format!("Invalid multipart part: {}", e))?;

       let form = reqwest::multipart::Form::new()
           .text("reqtype", "fileupload")
           .text("time", time_str)
           .part("fileToUpload", part);

       let resp = client
           .post("https://litterbox.catbox.moe/resources/internals/api.php")
           .multipart(form)
           .send()
           .await
           .map_err(|e| format!("Litterbox upload request failed: {}", e))?;

       if resp.status().is_success() {
           let url = resp.text().await.map_err(|e| e.to_string())?;
           let clean = url.trim().to_string();
           if clean.starts_with("http") {
               Ok(clean)
           } else {
               Err(format!("Litterbox returned error: {}", clean))
           }
       } else {
           Err(format!("Litterbox HTTP error {}", resp.status()))
       }
   }
   ```
4. Implement `ensure_media_vault_table` SQLite initialization in `repo_prompts.db`.
5. Enhance `download_telegram_photo` to upload downloaded photos to Catbox/Litterbox and return the compound URI.

---

### Recipe 2: Supabase-Free Fleet Identity & Reports
- **Target File**: `src-tauri/src/modules/telegram_inbound/reports_a.rs`
- **Objective**: Standardize the Telemetry Header across reports and provide full standalone host/instance/account inventory without Supabase.

#### Transformation Steps:
1. Standardize `format_fleet_identity_report()`:
   - Extract host machine (`COMPUTERNAME` / `HOSTNAME` / `detect_machine_name()`).
   - Extract local IP (`detect_local_ip()`).
   - Ingest registry (`instance::load_registry()`) and active instance details.
   - Query OS PIDs via `instance::find_pids_for_data_dir(&active_inst.data_dir, active_inst.is_default)`.
   - Ingest active account and extract dual-window quota (`4h` and `weekly`) via `auto_switcher::extract_dual_window_quotas`.
   - Render structured HTML card featuring the Telemetry Header:
     ```html
     🌐 <b>Standalone Fleet Identity Card</b>
     ━━━━━━━━━━━━━━━━━━━━━━━━
     🖥️ <b>Machine:</b> <code>{machine_name}</code> (<code>{local_ip}</code>)
     🏷️ <b>Instance:</b> <code>{active_inst_name}</code> (<code>{active_inst_id}</code>)
     👤 <b>Bound Account:</b> <code>{active_email}</code>
     🚦 <b>Status:</b> {running_tag} (PIDs: <code>{pid_str}</code>)
     📊 <b>Local Inventory:</b> {instances_count} instances | {accounts_count} accounts
     ```
2. Enhance `format_instances_report()`:
   - List each instance profile with running PID status emoji (`🟢` / `⚪`).
   - Monospace format data directories and bound account emails.
3. Enhance `format_accounts_report()`:
   - Display tier badges (`PRO`, `FREE`, `ULTRA`).
   - Format 4h rolling and weekly quota balances.

---

### Recipe 3: Telegram Notification Hub HTML Cards & Bounded Copy Buttons
- **Target File**: `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
- **Objective**: Render high-contrast visual cards, wrap prompt previews in `<blockquote expandable>`, resolve images, and construct bounded `copy_text` inline buttons.

#### Transformation Steps:
1. Wrap active prompt in `<blockquote expandable>`:
   ```rust
   let prompt_section = if !prompt_preview_text.is_empty() {
       format!(
           "\n💬 <b>Active Prompt:</b> (<code>{} words</code>)\n<blockquote expandable><pre><code class=\"language-markdown\">{}</code></pre></blockquote>\n",
           prompt_words_count,
           crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500)
       )
   } else {
       String::new()
   };
   ```
2. Safe copy text boundary invariant (clamped to <= 3,900 characters):
   ```rust
   let safe_prompt_copy = if full_prompt_content.len() > 3900 {
       let mut end = 3900;
       while end > 0 && !full_prompt_content.is_char_boundary(end) {
           end -= 1;
       }
       format!("{}...", &full_prompt_content[..end])
   } else {
       full_prompt_content.clone()
   };
   ```
3. Construct inline keyboard rows:
   ```rust
   let mut keyboard_rows = Vec::new();
   if !safe_prompt_copy.trim().is_empty() {
       keyboard_rows.push(vec![serde_json::json!({
           "text": "📋 Copy Prompt",
           "copy_text": {
               "text": safe_prompt_copy
           }
       })]);

       if !resolved_image_urls.is_empty() {
           let mut composite = safe_prompt_copy.clone();
           composite.push_str("\n\n---\n### Attached Images:\n");
           for url in &resolved_image_urls {
               composite.push_str(&format!("![image]({})\n", url));
           }
           let safe_composite = if composite.len() > 3900 {
               let mut end = 3900;
               while end > 0 && !composite.is_char_boundary(end) {
                   end -= 1;
               }
               format!("{}...", &composite[..end])
           } else {
               composite
           };

           keyboard_rows.push(vec![serde_json::json!({
               "text": "🖼️ Copy Prompt + Images",
               "copy_text": {
                   "text": safe_composite
               }
           })]);
       }
   }
   ```

---

### Recipe 4: Inbound Email Command Parser Extension
- **Target Files**:
  - `src-tauri/src/modules/email_inbound/parse_legacy.rs`
  - `src-tauri/src/modules/email_inbound/parse_pipe.rs`
  - `src-tauri/src/modules/email_inbound/handlers_b.rs`
- **Objective**: Enable inbound email commands `FLEET`, `WHOAMI`, `CLUSTER`, and `STATUS` to return complete node inventory.

#### Transformation Steps:
1. In `src-tauri/src/modules/email_inbound/parse_legacy.rs`:
   Add matching for `fleet`, `whoami`, `cluster`:
   ```rust
   if lower_subj == "fleet"
       || lower_subj == "whoami"
       || lower_subj == "cluster"
       || lower_subj == "machines"
       || lower_subj == "agm fleet"
       || lower_subj == "agm whoami"
   {
       return InboundAction::StatusQuery {
           target: "*".to_string(),
       };
   }
   ```
2. In `src-tauri/src/modules/email_inbound/parse_pipe.rs`:
   Add matching in pipe-delimited syntax:
   ```rust
   if lower_cmd == "fleet" || lower_cmd == "agm fleet" || lower_cmd == "whoami" {
       return Some(InboundAction::StatusQuery {
           target: target.to_string(),
       });
   }
   ```
3. In `src-tauri/src/modules/email_inbound/handlers_b.rs`:
   Update `handle_status_query` to format the standardized Telemetry Header and list all registered sandboxes and bound accounts.

---

## 3. Pre-flight Quality Verification & Git Hygiene

Before completing any subtask or committing code:

1. **Rust Pre-flight Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
2. **Targeted Unit Tests**:
   ```bash
   cd src-tauri && cargo test modules::telegram_inbound
   cd src-tauri && cargo test modules::notification_hub
   cd src-tauri && cargo test modules::email_inbound
   ```
3. **Clean Commit**:
   Stage only the touched files and commit using atomic GitMap CLI:
   ```bash
   gitmap cpf -m "feat(telemetry): telegram html cards, catbox media vault & standalone fleet discovery"
   ```

# Subtask 03: Free Image Storage, Multi-Tier Hosting & Reusable Media Assets

## Metadata
- **Subtask ID:** `03`
- **Slug:** `03-free-image-storage-and-reusable-assets`
- **Parent Plan:** `.ai-memory/plans/telegram-chat-enhancement-and-integration-solutions.md`
- **Status:** Pending
- **Owner:** Antigravity AI
- **Target Subsystem:** `src-tauri/src/modules/telegram_inbound/media.rs`

---

## 1. Objectives & Scope
Implement a resilient, zero-cost image lifecycle and reuse engine that allows users to send photos, screenshots, and diagrams into Telegram, automatically caches them locally, mirrors them to free public CDNs, indexes them in SQLite, and enables one-click copy and reuse in prompt injections without requiring paid cloud storage or API keys.

---

## 2. Target Files & Line Numbers

1. **`src-tauri/src/modules/telegram_inbound/media.rs`**
   - Lines 1-74: `download_telegram_photo(bot_token: &str, msg: &Value) -> Option<PathBuf>`
   - Lines 75-110: `upload_image_to_catbox(img_bytes: &[u8], filename: &str) -> Result<String, String>`
   - Lines 111+: New definitions for `send_telegram_photo`, `upload_image_to_litterbox`, `CompoundMediaUri`, `ensure_media_vault_table`, and `query_media_by_hash`.
2. **`src-tauri/src/modules/telegram_inbound/mod.rs`**
   - Export public media methods and `CompoundMediaUri`.
3. **`src-tauri/src/modules/repo_db.rs`**
   - Lines 120-175: Enhance `extract_image_payload_or_path` to parse `CompoundMediaUri` format.
4. **`src-tauri/src/modules/notification_hub/telegram_alerts.rs`**
   - Lines 128-164: Ingest `CompoundMediaUri` to populate `🖼️ Copy Prompt + Images` inline keyboard buttons.

---

## 3. Data Structures & Signatures

### 3.1 Compound Media URI Struct
```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundMediaUri {
    pub telegram_file_id: Option<String>,
    pub public_url: Option<String>,
    pub local_path: Option<String>,
}

impl CompoundMediaUri {
    pub fn parse(raw: &str) -> Self;
    pub fn format(&self) -> String;
    pub fn best_markdown_url(&self) -> Option<&str>;
}
```

### 3.2 Media Vault SQLite Record
```rust
#[derive(Debug, Clone)]
pub struct MediaVaultRecord {
    pub id: String,
    pub sha256: String,
    pub local_path: String,
    pub public_url: Option<String>,
    pub telegram_file_id: Option<String>,
    pub mime_type: String,
    pub is_active: bool,
    pub created_at: i64,
}
```

### 3.3 Function Signatures
```rust
/// Transmit image bytes directly to Telegram chat, acquiring permanent file_id
pub async fn send_telegram_photo(
    bot_token: &str,
    chat_id: i64,
    img_bytes: &[u8],
    caption: Option<&str>,
) -> Result<String, String>;

/// Upload image to Catbox.moe free public CDN (permanent retention)
pub async fn upload_image_to_catbox(
    img_bytes: &[u8],
    filename: &str,
) -> Result<String, String>;

/// Fallback ephemeral upload to Litterbox (72h retention)
pub async fn upload_image_to_litterbox(
    img_bytes: &[u8],
    filename: &str,
    time_hours: u32,
) -> Result<String, String>;

/// High-level file upload wrapper with SHA256 caching in media_vault
pub async fn upload_local_image_file(
    path: &std::path::Path,
) -> Result<String, String>;
```

---

## 4. Implementation Details

1. **Multi-Tier Free Hosting Ladder**:
   - **Tier 1 (Catbox.moe)**: Direct multipart upload to `https://catbox.moe/user/api.php` (`reqtype=fileupload`, `fileToUpload=<part>`).
   - **Tier 2 (Litterbox Fallback)**: If Catbox returns a 5xx or times out after 15s, execute fallback to `https://litterbox.catbox.moe/resources/internals/api.php` (`reqtype=fileupload`, `time=72h`).
   - **Tier 3 (Telegram Bot API)**: Direct transmission via `https://api.telegram.org/bot<token>/sendPhoto` to acquire Telegram cloud's native `file_id`.
2. **SQLite Deduplication & Caching (`media_vault`)**:
   - Compute SHA-256 hash before upload.
   - If hash exists in `media_vault` with a valid `public_url`, immediately return existing URL without network upload.
   - On upload success, persist row with `is_active = 1` and timestamp.
3. **Dual-Mode URI Assembly**:
   - Assemble `tg:<file_id>|catbox:<public_url>|local:<local_path>` for prompt injection references.

---

## 5. Acceptance Criteria

- [ ] `send_telegram_photo` sends photos to Telegram chat and extracts permanent `file_id`.
- [ ] `upload_image_to_catbox` returns permanent HTTPS URLs (`https://files.catbox.moe/...`) without API keys.
- [ ] `upload_image_to_litterbox` acts as an ephemeral fallback if Catbox is unreachable.
- [ ] `CompoundMediaUri::parse` and `format` correctly handle compound and legacy URL strings.
- [ ] `media_vault` table in `repo_prompts.db` prevents redundant uploads of identical SHA-256 hashes.
- [ ] Only positive booleans (`is_active`, `is_success`, `has_images`) are utilized.
- [ ] No bare errors swallowed; all fallible calls use `AppError` or `crate::error::record_ignored` with justification.

---

## 6. Verification Steps

1. **Unit Testing**:
   - Create tests in `src-tauri/src/modules/telegram_inbound/` testing `CompoundMediaUri::parse` on:
     - Compound: `tg:ABC123_xyz|catbox:https://files.catbox.moe/123.png|local:/tmp/image.png`
     - Pure URL: `https://files.catbox.moe/test.png`
     - Pure local: `local:/tmp/test.png`
2. **Lint & Preflight**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   cd src-tauri && cargo fmt -- --check
   ```

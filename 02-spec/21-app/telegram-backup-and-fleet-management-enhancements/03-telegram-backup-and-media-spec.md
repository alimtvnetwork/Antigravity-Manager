# Telegram Prompt Backup & Media Forwarding Specification

**Task Slug**: `telegram-backup-and-fleet-management-enhancements`  
**Location**: `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/03-telegram-backup-and-media-spec.md`  

---

## 1. Telegram Prompt Backup & Restore Commands

### 1.1 Bot Command Registration
- Register `/backup` and `/restore` in `register_telegram_bot_commands` in `src-tauri/src/modules/telegram_inbound/config.rs`:
  - `{ "command": "backup", "description": "Backup active running prompts & attached images" }`
  - `{ "command": "restore", "description": "Restore & re-inject prompt queue into target instance" }`

### 1.2 Interactive Inline Keyboard Affordances
- When `/backup` or `/backup prompts` is executed:
  - Return the backup report with interactive inline buttons:
    - `[ 📦 Backup Prompts ]`
    - `[ ♻️ Restore Prompts ]`
    - `[ 🖼️ Send with Images ]`
    - `[ 📋 List Backups ]`
- When `/restore` is executed:
  - Re-inject queued prompts into target instance (FIFO order) and report restored count and attached image counts.

### 1.3 Sending Prompts with Images via Telegram
- Parse attached photos or document images sent to the Telegram bot.
- Save attached media into temporary workspace attachments or base64 data payloads.
- Inject prompts with attached images into the target instance using `submit_prompt_via_cdp` or external CLI worker.

---

## 2. Temporary File Service Architecture (Future Proofing)

### 2.1 Short-Lived Image Upload Endpoint Design
- Design REST endpoint `/api/media/upload-temporary`:
  - Accepts multipart/form-data or binary image stream with TTL header (e.g. `X-Media-TTL: 3600`).
  - Stores image in local cache directory `~/.antigravity/temp_media/<hash>.<ext>`.
  - Returns a temporary local/reverse-proxied URL: `http://127.0.0.1:port/api/media/temp/<hash>`.
  - Prompts referencing this URL allow remote or cross-process consumers to fetch images seamlessly without transferring bulky Base64 payloads over CLI arguments.

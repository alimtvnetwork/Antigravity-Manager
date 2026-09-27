# Subtask 02: Telegram Message Chunking & 4096-Character Limit Protection

## Metadata
- **Parent Plan:** `77-installer-fork-fix-and-telegram-fleet-nodes.md`
- **Status:** Completed
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound.rs`

---

## 1. Description
Telegram Bot API rejects payloads larger than 4096 characters with `Bad Request: message is too long`.
1. Implement `send_telegram_message_chunked(bot_token: &str, chat_id: i64, text: &str)` in `telegram_inbound.rs`.
2. Break messages > 4000 characters into sequential chunks at `\n` boundaries.
3. Transmit sequential chunks with 80ms delay.
4. Replace direct calls to `send_telegram_message` with `send_telegram_message_chunked`.

---

## 2. Verification Criteria
- [x] Unit test validating string splitting for messages > 4000 characters.
- [x] Successful compilation without warning.

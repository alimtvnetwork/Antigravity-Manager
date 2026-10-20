# Subtask 04: Telegram Backup, Restore & Media Enhancements

**Subtask ID**: `sub-04-telegram-media`  
**Parent Plan**: `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
**Status**: PENDING  

## Objectives
1. Telegram commands registration:
   - Register `/backup` and `/restore` in `register_telegram_bot_commands` in `src-tauri/src/modules/telegram_inbound/config.rs`.
2. Interactive inline keyboard:
   - Provide inline action buttons on backup and restore reports: `[ 📦 Backup Prompts ]`, `[ ♻️ Restore Prompts ]`, `[ 🖼️ Send with Images ]`.
3. Support media attachments when dispatching prompts via Telegram.
4. Document short-lived file upload endpoint design for future cloud/reverse-proxied image hosting.

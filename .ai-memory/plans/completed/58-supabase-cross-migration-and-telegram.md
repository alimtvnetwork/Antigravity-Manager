# Consolidated Plan 58: Supabase Cross-Database Failover Migration, Table Verification & Telegram Inbound Daemon

> **Execution Lifecycle:**
> - Started: User request for in-app Supabase table verification, cross-database failover migration (transferring recent critical state from damaged to new endpoint), Telegram inbound bot command daemon, and formatted snapshot instructions.
> - Completed in: Continuous loop.
> - Total Subtasks Completed: 5/5
> - Verification Gates: `npx tsc --noEmit` passed (exit 0), `cargo fmt` passed (exit 0).

---

## 1. Executive Summary

This implementation delivers the final operational capabilities for the Supabase Split-DB and remote orchestration ecosystem:
1. **In-App Database Table Verification (`check_supabase_endpoint_tables`):**
   - Probes endpoints via PostgREST to verify whether required tables exist without requiring administrative Postgres catalog permissions.
   - Endpoint cards in Settings render green badges for verified tables (`✓ nodes`, `✓ workspace_leases`, etc.) and warning badges for missing tables (`✗ command_queue`).
2. **Cross-Supabase Selective Data Migration (`migrate_supabase_data`):**
   - Enables seamless disaster recovery / failover: when a Supabase project is damaged or reaching limits, active state is copied to a fresh target project.
   - Selectively migrates active online nodes, active instance profiles, unexpired workspace leases, and the last 50 command records.
   - Prevents log bloat from transferring over to the new target, guaranteeing immediate free-tier compliance.
3. **Telegram Bot Inbound Watcher & Command Poller Daemon (`telegram_inbound.rs`):**
   - Background daemon polling the Telegram Bot API (`getUpdates`) with configurable bot token and optional allowed chat ID security filter.
   - Automatically parses:
     - `How many machines are running?` / `/snapshot` -> Replies with active cluster nodes, IPs, uptimes, and active profiles.
     - `FF` / `/ff` -> Triggers double-play workspace profile fast-forward rotation.
     - `CMD:<node-alias>:<command>` -> Enqueues terminal command into Supabase Secondary DB (or local execution).
4. **Enhanced Remote Command Formats in Snapshot Replies:**
   - Both Email and Telegram snapshot replies now include clear, copyable command format examples: `CMD:<Node-Alias>:<PowerShell-Command>`, `FF`, and `SNAPSHOT`.
5. **Settings UI Management:**
   - "Migrate to New DB" modal in toolbar.
   - "Check Tables" button on each endpoint card with live badges.
   - "Telegram Inbound Bot Integration" card with token input, allowed chat ID, connection test, test ping dispatch, and live polling indicator.

---

## 2. Completed Subtasks Log

### Subtask 01: In-App Database Table Verification
- **Target Files:** `src-tauri/src/modules/supabase_client.rs`, `src-tauri/src/commands/supabase.rs`, `src/services/supabaseService.ts`
- **Delivered:** `check_table_exists` and `verify_expected_tables` in `supabase_client.rs`, exposed via IPC command `check_supabase_endpoint_tables`. TypeScript binding in `supabaseService.ts`.

### Subtask 02: Cross-Supabase Database Data Migration Engine
- **Target Files:** `src-tauri/src/modules/supabase_sync.rs`, `src-tauri/src/commands/supabase.rs`, `src/services/supabaseService.ts`
- **Delivered:** `migrate_database_data` in `supabase_sync.rs` copying active nodes, running profiles, unexpired leases, and last 50 commands. Exposed via IPC command `migrate_supabase_data`.

### Subtask 03: Telegram Bot Inbound Watcher & Command Poller Daemon
- **Target Files:** `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/commands/telegram.rs`, `src-tauri/src/modules/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src/services/telegramService.ts`
- **Delivered:** Async long-polling daemon over Telegram Bot API with getUpdates, message dispatch, snapshot generation, and reply delivery.

### Subtask 04: Enhanced Cluster Snapshot & Remote Command Format Replies
- **Target Files:** `src-tauri/src/modules/email_inbound.rs`, `src-tauri/src/modules/telegram_inbound.rs`
- **Delivered:** Standardized remote command format guidance (`CMD:<Node-Alias>:<Command>`, `FF`, `SNAPSHOT`) included in cluster snapshot responses.

### Subtask 05: Telegram Configuration & Supabase Cross-Migration UI
- **Target Files:** `src/components/settings/SupabaseSyncSettings.tsx`
- **Delivered:** Added "Migrate to New DB" toolbar button & modal, "Check Tables" verification badges on endpoint cards, and full Telegram Bot integration card.

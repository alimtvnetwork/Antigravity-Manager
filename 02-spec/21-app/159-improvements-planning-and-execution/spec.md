# Specification: 30 Architecture, Gateway, Database & UI Improvements (61–90)

## 1. Executive Summary

This document specifies the execution plan for the **next 30 system, gateway, database, and UI improvements (Improvements 61–90)** across Antigravity-Manager, continuing directly from Improvements 1–30 (`157`) and Improvements 31–60 (`158`).

### 1.1 Architecture Domains & Subtasks
1. **Batch 1 (Imp 61–66)**: Multi-Instance Profile Sandboxing & Process Hygiene
2. **Batch 2 (Imp 67–72)**: Remote Daemons, Telegram Health & IMAP Email Hardening
3. **Batch 3 (Imp 73–78)**: Split-SQLite WAL Auto-Checkpointing, FTS & Integrity
4. **Batch 4 (Imp 79–84)**: Reverse Proxy Protocol Adapters & Multimodal Streaming
5. **Batch 5 (Imp 85–90)**: React Desktop UI, Accessibility & System Diagnostics

### 1.2 Execution Constraints
- **Zero Testing / Zero Build Checking Mode**: Per user instructions, no tests or build checks are run during execution (`do not do the testing now. For now, just proceed. Do not do the build checking or anything`).
- **Strictly Relative Git Paths**: All file paths referenced in code and specs must be repository-relative (`src/...`, `src-tauri/...`, `02-spec/...`, `.ai-memory/...`).
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Sequential Completion**: Each task is executed and immediately moved to the completed state.

---

## 2. Exhaustive Index of Improvements 61–90

| # | Improvement Title | Target Subsystem | Target Files | Technical Goal |
| :--- | :--- | :--- | :--- | :--- |
| **61** | Stale Session Lock Recovery | Sandboxing | `src-tauri/src/modules/instance/lifecycle.rs` | Auto-remove orphaned `.vscode-server` and `.antigravity-session` lockfiles when holding PID is dead. |
| **62** | Sandboxed Keyring Bypass Marker | Sandboxing | `src-tauri/src/modules/instance/launch.rs` | Enforce `--password-store=basic` and `--use-mock-keychain` across multi-instance launches. |
| **63** | Instance Launch Backoff Ladder | Sandboxing | `src-tauri/src/modules/instance/lifecycle.rs` | Implement bounded retry backoff (500ms, 1000ms, 2000ms) when probing instance CDP ports. |
| **64** | Deep-Merge Telemetry Scrubbing | Sandboxing | `src-tauri/src/modules/instance/clone.rs` | Sanitize `storage.json` telemetry identifiers (`telemetry.machineId`, `telemetry.macMachineId`). |
| **65** | Cross-Platform Shell Escaper | Utilities | `src-tauri/src/utils/command.rs` | Provide safe whitespace and special character argument escaping across Windows and Unix shells. |
| **66** | Conscious PID Verification | Process Hygiene | `src-tauri/src/modules/process_cleaner.rs` | Verify process executable identity and parent-child hierarchy before sending SIGKILL. |
| **67** | Telegram Confirmation Callbacks | Remote Daemon | `src-tauri/src/modules/telegram_inbound/send_flow.rs` | Add inline callback verification (`[Confirm]` / `[Cancel]`) for destructive actions. |
| **68** | Telegram Fast Ping Command | Remote Daemon | `src-tauri/src/modules/telegram_inbound/daemon.rs` | Sub-50ms diagnostic ping reporting bot uptime, active instances, and SQLite connection status. |
| **69** | Telegram Rate-Limited Queue Pacer | Remote Daemon | `src-tauri/src/modules/telegram_inbound/formatting.rs` | Token bucket pacer (max 30 req/sec) to eliminate Telegram 429 rate limit errors. |
| **70** | Inbound Email MIME Parser Polish | Remote Daemon | `src-tauri/src/modules/email_inbound/imap.rs` | Strip tracking pixels, trailing signatures, and HTML boilerplate from inbound prompt bodies. |
| **71** | IMAP Idle Keepalive Heartbeat | Remote Daemon | `src-tauri/src/modules/email_watcher/heartbeat.rs` | Send structured NOOP / IDLE keepalives every 15 minutes to keep IMAP connection alive. |
| **72** | Multi-Channel Alert Matrix | Notification Hub | `src-tauri/src/modules/notification_hub/mod.rs` | Support routing alert severities (Critical, Warning, Info) to specific channels. |
| **73** | WAL Concurrency Checkpointer | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Provide `checkpoint_split_db(conn, mode)` executing `PRAGMA wal_checkpoint(PASSIVE)`. |
| **74** | PRAGMA Optimize Idle Maintenance | Split-SQLite | `src-tauri/src/modules/db_orm/connection.rs` | Run `PRAGMA optimize;` to update SQLite query planner statistics across split databases. |
| **75** | Strict FIFO Query Invariant Guard | Split-SQLite | `src-tauri/src/modules/repo_db/queue.rs` | Enforce `ORDER BY created_at ASC, id ASC` across all prompt restore, dispatch, and queue processing. |
| **76** | Transactional Rollback Protection | Split-SQLite | `src-tauri/src/modules/repo_db/prompts_crud.rs` | Wrap multi-row prompt status and sequence updates in atomic SQLite transactions. |
| **77** | Database Slow Query Telemetry | Split-SQLite | `src-tauri/src/modules/speed_errors.rs` | Track slow database operations (>100ms) with `E2002` diagnostic records in `errors.db`. |
| **78** | Foreign Key Cascade Integrity | Split-SQLite | `src-tauri/src/modules/repo_db/schema.rs` | Ensure `PRAGMA foreign_keys = ON;` is consistently enforced to prevent orphaned references. |
| **79** | Stream Chunk Deduplication Filter | Reverse Proxy | `src-tauri/src/proxy/handlers/openai/chat_completions_success.rs` | Discard duplicate `data: [DONE]` markers and empty newline bursts emitted by upstream adapters. |
| **80** | Thinking Block Graceful Healing | Reverse Proxy | `src-tauri/src/proxy/thinking_store/mod.rs` | Backfill empty thought blocks with valid syntax when upstream reasoning payloads arrive. |
| **81** | Token Budget Sliding Window | Reverse Proxy | `src-tauri/src/proxy/server/admin_prompts.rs` | Implement an in-memory sliding window rate limiter per client API key. |
| **82** | OpenAI to Gemini Tool Translator | Reverse Proxy | `src-tauri/src/proxy/mappers/openai/tools.rs` | Normalize parameter types (`string`, `integer`, `array`) in function declarations. |
| **83** | Egress Proxy Failover Retries | Reverse Proxy | `src-tauri/src/proxy/proxy_pool/proxypoolmanager_impl.rs` | Fall back to next healthy proxy if egress connection resets during initial TCP handshake. |
| **84** | Downstream SSE Flow Control Cap | Reverse Proxy | `src-tauri/src/proxy/handlers/openai/chat_completions.rs` | Bound downstream buffer size to prevent memory exhaustion on stalled clients. |
| **85** | React Error Boundary Fallbacks | Desktop UI | `src/components/common/ErrorBoundary.tsx` | Provide contextual, non-intrusive dark-glass recovery cards with retry buttons. |
| **86** | Virtualized Tree Node Threshold | Desktop UI | `src/components/instances/PromptTreeView/TreePanel.tsx` | Optimize rendering for conversations with 50+ turns by limiting offscreen DOM nodes. |
| **87** | Semantic Design Token Harmonization | Desktop UI | `src/index.css` | Standardize dark-glass variables (`--app-bg`, `--app-surface`, `--app-border`) with RGB fallbacks. |
| **88** | Clipboard Dual-MIME Extraction | Desktop UI | `src/utils/clipboard.ts` | Centralize rich HTML formatting with plain text fallback across operating systems. |
| **89** | Keyboard Shortcut Registry | Desktop UI | `src/utils/keyboardShortcuts.ts` | Centralized keyboard event manager handling modal layers and input focus isolation. |
| **90** | Comprehensive System Diagnostics | Telemetry | `src-tauri/src/commands/doctor.rs` | Unified health check command (`agm doctor`) auditing DB integrity, proxy, and processes. |

# Master Execution Plan: 229-telegram-onboarding-and-prompt-sending-issue

- **Parent Task Slug**: `telegram-onboarding-and-prompt-sending-issue`
- **Parent Task Sequence**: `229`
- **Target Release**: Patch/Bugfix Optimization
- **Status**: COMPLETED
- **Completed**: 2026-10-27
- **Verification Evidence**:
  - `cargo fmt -- --check`: Passed (code 0)
  - `cargo check`: Passed (code 0, 1m 01s)
  - `npm run build`: Passed (code 0, 22.28s)
  - Subtask-01: DONE (sub-182749-subtask-01)
  - Subtask-02: DONE (sub-182755-subtask-02)
  - Subtask-03: DONE (sub-182758-subtask-03)
- **Prompt Hash**: Verified
- **Created**: 2026-10-27

---

## 1. Executive Summary & Intent
This plan orchestrates comprehensive architectural and operational optimizations for the Telegram integration in Antigravity Manager, slashing command and prompt response latency to sub-2 seconds (`1–2s`), resolving onboarding bottlenecks, and decoupling background broadcasting:
1. **Eliminate 60s/30s Polling Sleep Latency**: Replace hardcoded disabled/idle sleep loops in `src-tauri/src/modules/telegram_inbound/daemon.rs` with a reactive `tokio::sync::Notify` barrier (`TELEGRAM_DAEMON_NOTIFIER`). Saving configuration or connecting via CLI wakes the polling daemon in <5ms.
2. **Concurrent Inbound Update Dispatch**: Refactor `daemon.rs` so individual command execution (`deliver_ui_or_command`) runs in spawned background Tokio tasks rather than blocking the sequential `getUpdates` receiver loop. Update offsets are acknowledged immediately.
3. **Instant Prompt Queue Wakeup**: In `src-tauri/src/modules/telegram_inbound/injection.rs`, integrate `crate::modules::scheduler::notify_prompt_queue()` and `repo_db::check_and_dispatch_enqueued_prompts(Some(&resolved_instance))` so prompts arriving from Telegram wake the queue worker instantly instead of waiting up to 30 seconds for the idle backoff ticker.
4. **Fast Token Verification (< 1.5s)**: In `src-tauri/src/modules/telegram_inbound/config.rs`, decouple `register_telegram_bot_commands` from `test_telegram_connection`. `test_telegram_connection` returns immediately after `getMe` succeeds (< 200ms), while `setMyCommands` runs in a detached background task.
5. **Webhook Conflict Elimination (`deleteWebhook`)**: Implement `delete_telegram_webhook` and invoke it automatically prior to daemon polling and upon encountering HTTP 409 Conflict, permanently eliminating the silent freeze when bots transition from webhooks.
6. **Pooled HTTP Client**: Introduce `TELEGRAM_HTTP_CLIENT` (`Lazy<reqwest::Client>`) with persistent connection pooling and TCP keepalive across daemon polling, messaging, detection, and broadcasting, eliminating per-request TCP/TLS handshake overhead.
7. **Concurrent Broadcasting Fan-out**: In `src-tauri/src/modules/notification_hub/channels.rs`, execute Email, Telegram, and Supabase channel dispatches concurrently via `tokio::join!`, preventing slow SMTP connections from delaying Telegram alerts.
8. **Frontend Auto-Save & Onboarding Flow Polish**: Automatically set `is_enabled = true` and save configuration when chat auto-detection succeeds in `src/pages/Telegram.tsx` and `useTelegramHandlers.ts`.
9. **Telegram Improvements & Suggestions Specification**: Author a detailed catalog of 15+ architectural improvements for future Telegram integration.

---

## 2. Work Breakdown & Subtask Waves

```text
Wave 1 (Planning & Spec Step):
  ├── Lead: Parent Plan & Master Roadmap
  ├── Spec Author 01: 01-architecture-spec.md & 01-subtask-architecture-and-contracts.md
  ├── Spec Author 02: 02-component-spec.md & 02-subtask-inbound-and-polling.md
  └── Spec Author 03: 03-migration-and-verification-spec.md, 04-telegram-improvements-and-suggestions-spec.md, 03-subtask-broadcasting-and-onboarding.md

Wave 2 (Backend Inbound Polling & Prompt Queue Execution):
  ├── Worker 01: daemon.rs Reactive Wakeup (TELEGRAM_DAEMON_NOTIFIER) & Concurrent Dispatch
  ├── Worker 02: injection.rs Instant Queue Wakeup (notify_prompt_queue) & FIFO Integration
  └── Worker 03: config.rs Detached Commands, deleteWebhook & Pooled HTTP Client

Wave 3 (Broadcasting, Onboarding & Frontend Polish):
  ├── Worker 01: notification_hub Concurrent Channel Dispatch (tokio::join!)
  ├── Worker 02: detection.rs 409 Self-Healing & Webhook Cleanup
  └── Worker 03: Telegram.tsx & useTelegramHandlers.ts Auto-Enable & Auto-Save

Wave 4 (Verification & Quality Gates):
  └── Lead: E2E Verification, Cargo Check, Frontend Build & Atomic GitMap Commit
```

---

## 3. Disjoint File Bounding Boxes
- **Spec Author 01**: `02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/01-architecture-spec.md`, `.ai-memory/plans/subtasks/229-telegram-onboarding-and-prompt-sending-issue/01-subtask-architecture-and-contracts.md`
- **Spec Author 02**: `02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/02-component-spec.md`, `.ai-memory/plans/subtasks/229-telegram-onboarding-and-prompt-sending-issue/02-subtask-inbound-and-polling.md`
- **Spec Author 03**: `02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/03-migration-and-verification-spec.md`, `02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/04-telegram-improvements-and-suggestions-spec.md`, `.ai-memory/plans/subtasks/229-telegram-onboarding-and-prompt-sending-issue/03-subtask-broadcasting-and-onboarding.md`
- **Worker 01 (Inbound & Polling)**: `src-tauri/src/modules/telegram_inbound/daemon.rs`, `src-tauri/src/commands/telegram.rs`
- **Worker 02 (Prompt Injection & Queue)**: `src-tauri/src/modules/telegram_inbound/injection.rs`, `src-tauri/src/modules/notification_hub/channels.rs`
- **Worker 03 (Config, Webhook & Frontend)**: `src-tauri/src/modules/telegram_inbound/config.rs`, `src-tauri/src/modules/telegram_inbound/detection.rs`, `src/pages/Telegram.tsx`, `src/components/settings/EmailNotifications/useTelegramHandlers.ts`

---
plan: 229-telegram-onboarding-and-prompt-sending-issue
subtask: "01"
title: Telegram Daemon Reactive Wakeup Architecture, HTTP Connection Pooling & Inbound Concurrency Contracts
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/telegram_inbound/mod.rs
  - src-tauri/src/modules/telegram_inbound/daemon.rs
  - src-tauri/src/commands/telegram.rs
status: pending
---

# Subtask 01: Telegram Daemon Reactive Wakeup Architecture, HTTP Connection Pooling & Inbound Concurrency Contracts

## 1. Context & Motivation

In Antigravity Manager, remote developers interact with their workstation and cluster fleets via Telegram bots. However, two primary operational bottlenecks have severely degraded the responsiveness and onboarding user experience:
1. **60-Second Polling Sleep Trap**: When the Telegram daemon is disabled or lacks a configured token, `start_telegram_daemon` in `src-tauri/src/modules/telegram_inbound/daemon.rs` sleeps unconditionally for 60 seconds (or 30 seconds when the token is empty). Calling `save_telegram_config` or toggling the bot to enabled fails to break this sleep because the thread is already flagged as running, stalling the bot for up to a minute before the first poll executes.
2. **Unpooled Ephemeral HTTP Client Overhead**: Every outbound request (polling `getUpdates`, sending "typing" actions, answering callback queries, and delivering message replies) builds a fresh `reqwest::Client`. Each connection suffers a full TCP 3-way handshake and TLS 1.3 negotiation (300–600ms overhead per call).
3. **Sequential Inbound Head-of-Line Blocking**: Updates returned by `getUpdates` are executed serially within the single polling loop. If one command involves slower operations (capturing a screenshot, querying CLI telemetry, or uploading media), all subsequent updates and offset acknowledgments are queued and blocked.

This subtask defines the architecture contracts, data structures, and implementation steps to introduce `TELEGRAM_DAEMON_NOTIFIER` (`tokio::sync::Notify`), `TELEGRAM_HTTP_CLIENT` (`Lazy<reqwest::Client>`), and concurrent inbound update execution in `daemon.rs` and `commands/telegram.rs`.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Functions | Planned Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/daemon.rs` | `TELEGRAM_DAEMON_NOTIFIER`, `notify_telegram_daemon`, `TELEGRAM_HTTP_CLIENT`, `start_telegram_daemon`, `stop_telegram_daemon`, `answer_callback_query`, `send_chat_action`, `poll_telegram_updates_once` | 1. Define global `pub static TELEGRAM_DAEMON_NOTIFIER: Lazy<tokio::sync::Notify>`.<br/>2. Expose `pub fn notify_telegram_daemon()` to wake the polling loop immediately.<br/>3. Define `pub static TELEGRAM_HTTP_CLIENT: Lazy<reqwest::Client>` with connection pooling (90s idle timeout, keep-alive 30s).<br/>4. Refactor `start_telegram_daemon` loop using `tokio::select!` on idle/disabled sleeps.<br/>5. Decouple update execution into spawned Tokio tasks to prevent head-of-line blocking.<br/>6. Replace ephemeral `reqwest::Client::builder()` allocations with `TELEGRAM_HTTP_CLIENT`. |
| `src-tauri/src/modules/telegram_inbound/mod.rs` | Re-exports | Ensure `notify_telegram_daemon`, `TELEGRAM_DAEMON_NOTIFIER`, and `TELEGRAM_HTTP_CLIENT` are re-exported cleanly via `pub use daemon::*;`. |
| `src-tauri/src/commands/telegram.rs` | `save_telegram_config` | Invoke `telegram_inbound::notify_telegram_daemon()` on configuration save to wake the polling daemon immediately in < 5ms. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Implement `TELEGRAM_DAEMON_NOTIFIER` & `notify_telegram_daemon`

In `src-tauri/src/modules/telegram_inbound/daemon.rs`:
1. Import `tokio::sync::Notify` and `once_cell::sync::Lazy`.
2. Define the static notifier and public helper function:
   ```rust
   use once_cell::sync::Lazy;
   use tokio::sync::Notify;

   /// Global notifier to wake the Telegram daemon immediately from sleep traps
   pub static TELEGRAM_DAEMON_NOTIFIER: Lazy<Notify> = Lazy::new(Notify::new);

   /// Signal the Telegram daemon to break any idle/disabled sleep immediately
   pub fn notify_telegram_daemon() {
       TELEGRAM_DAEMON_NOTIFIER.notify_one();
   }
   ```
3. In `start_telegram_daemon()`, if `is_already_running` is true, invoke `notify_telegram_daemon()` before returning so that redundant start invocations still awaken any sleeping loop:
   ```rust
   pub fn start_telegram_daemon() {
       let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
       if is_already_running {
           notify_telegram_daemon();
           return;
       }
       // ...
   ```

### 3.2 Step 2: Implement Persistent `TELEGRAM_HTTP_CLIENT`

In `src-tauri/src/modules/telegram_inbound/daemon.rs`:
1. Define the persistent client with aggressive keep-alive and connection pooling:
   ```rust
   /// Global pooled HTTP client for Telegram operations, reusing TCP/TLS connections
   pub static TELEGRAM_HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
       reqwest::Client::builder()
           .pool_max_idle_per_host(10)
           .pool_idle_timeout(Duration::from_secs(90))
           .tcp_keepalive(Duration::from_secs(30))
           .connect_timeout(Duration::from_secs(10))
           .timeout(Duration::from_secs(35))
           .build()
           .unwrap_or_else(|e| {
               tracing::error!("[TelegramClient] Failed to build pooled client: {e}");
               reqwest::Client::new()
           })
   });
   ```
2. Refactor all auxiliary calls in `daemon.rs` to consume `TELEGRAM_HTTP_CLIENT`:
   - `answer_callback_query(token: &str, cb_id: &str)` (drops ephemeral client argument).
   - `send_chat_action(token: &str, chat_id: i64, action: &str)`.
   - `poll_telegram_updates_once()` (reuses `TELEGRAM_HTTP_CLIENT` instead of calling `reqwest::Client::builder().build()`).

### 3.3 Step 3: Refactor `start_telegram_daemon` with `tokio::select!` and Task Spawning

In `src-tauri/src/modules/telegram_inbound/daemon.rs`:
1. Replace static sleep loops with `tokio::select!`:
   ```rust
   if !is_enabled_flag {
       {
           let mut st = LAST_TELEGRAM_STATUS.write().await;
           st.is_running = false;
       }
       tokio::select! {
           _ = tokio::time::sleep(Duration::from_secs(60)) => {},
           _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
               tracing::info!("[TelegramDaemon] Reactive wakeup from disabled state");
           }
       }
       continue;
   }

   if clean_token.is_empty() {
       tokio::select! {
           _ = tokio::time::sleep(Duration::from_secs(30)) => {},
           _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
               tracing::info!("[TelegramDaemon] Reactive wakeup from empty-token state");
           }
       }
       continue;
   }
   ```
2. In the update ingestion loop, decouple update execution from the long-poll receiver:
   - Synchronously advance `last_update_id = max(last_update_id, up_id)`.
   - Synchronously perform authorization check (`chat_id == allowed_chat_id`).
   - Immediately fire `answer_callback_query` and `send_chat_action("typing")`.
   - Detach command execution via `tauri::async_runtime::spawn`:
     ```rust
     let token_clone = clean_token.clone();
     let media_clone = saved_media_opt.clone();
     let text_clone = text_raw.to_string();
     tauri::async_runtime::spawn(async move {
         let delivered = deliver_ui_or_command(&token_clone, chat_id, &text_clone, &media_clone).await;
         if delivered.is_none() {
             tracing::debug!("[TelegramDaemon] Command produced no direct reply for chat {}", chat_id);
         }
     });
     ```
   - This ensures `start_telegram_daemon` can immediately yield and start the next `getUpdates` poll without waiting for command execution to finish.

### 3.4 Step 4: Wire Reactive Wakeup in `src-tauri/src/commands/telegram.rs`

In `src-tauri/src/commands/telegram.rs`:
Update `save_telegram_config` to invoke `telegram_inbound::notify_telegram_daemon()`:
```rust
#[tauri::command]
pub async fn save_telegram_config(config: TelegramConfig) -> AppResult<()> {
    let is_enabled = config.is_enabled;
    telegram_inbound::save_config(&config)?;

    if is_enabled {
        telegram_inbound::start_telegram_daemon();
        telegram_inbound::notify_telegram_daemon();
    } else {
        telegram_inbound::stop_telegram_daemon();
        telegram_inbound::notify_telegram_daemon();
    }
    Ok(())
}
```

---

## 4. Error Handling & Invariant Enforcement

1. **No Silent Errors Rule**:
   - Any fallible operations that fail must route their error into `AppError` or be explicitly logged with justification via `crate::error::record_ignored(...)`.
   - Never use bare `let _ = ...` on network operations.
2. **Best-Effort Operations Justification**:
   - `answer_callback_query` and `send_chat_action` must include comment:
     `// Justification: best-effort telegram user experience feedback; failure logged without disrupting core command execution`
3. **FIFO Monotonicity Invariant**:
   - `ORDER BY created_at ASC, id ASC` is preserved across all queue-related operations.
   - Update offset acknowledgment ensures updates are processed once and never duplicated.
4. **Thread Safety**:
   - `TELEGRAM_DAEMON_NOTIFIER.notify_one()` is lock-free and thread-safe across asynchronous task boundaries.

---

## 5. Verification Criteria & Test Assertions

1. **Instant Daemon Wakeup Verification**:
   - Start daemon in disabled state (`is_enabled: false`).
   - Confirm status reports `is_running: false`.
   - Call `save_telegram_config` with `is_enabled: true` and valid bot token.
   - Assert daemon begins polling and `is_running: true` within **< 50ms**, without waiting for the 60s timeout.
2. **HTTP Connection Reuse Verification**:
   - Send two consecutive requests (e.g. typing indicator followed by message reply).
   - Assert that no new TCP handshakes or TLS negotiations occur, and roundtrip time drops by > 300ms.
3. **Concurrent Update Processing Verification**:
   - Dispatch a slow command (e.g. screenshot or status query) followed immediately by `/ping`.
   - Assert `/ping` reply is generated concurrently without waiting for the screenshot to finish.
4. **Code Quality Gates**:
   - Zero compilation errors.
   - Zero `cargo clippy` warnings.
   - Formatting compliant with `cargo fmt`.

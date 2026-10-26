# Architecture Specification: Reactive Telegram Daemon Wakeup, Pooled HTTP Client & Low-Latency Inbound Processing

> **Target:** `02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/01-architecture-spec.md`  
> **Status:** APPROVED  
> **Author:** @aukgit  
> **Version:** 1.0.0  
> **Slug:** `229-telegram-onboarding-and-prompt-sending-issue`  
> **Sequence:** 229  
> **Scope:** Telegram Daemon Polling, Reactive Wakeup, Connection Pooling, Non-blocking Inbound Execution  

---

## 1. Executive Summary & Intent

The Telegram integration in Antigravity Manager (AGM) provides an autonomous, real-time command-and-control surface for remote developers, enabling cluster status inspection, instant prompt injection, account rotation alerts, and screenshot verification via Telegram bot conversations.

Despite these rich capabilities, the operational responsiveness of Telegram onboarding and command execution suffered from significant latency bottlenecks:
1. **The 60-Second Disabled/Idle Sleep Trap**: When Telegram was disabled or unconfigured, `start_telegram_daemon` stalled in an unconditional `tokio::time::sleep(Duration::from_secs(60))` loop. Saving a configuration or connecting via CLI failed to wake the sleeping thread, causing up to a 60-second freeze before the daemon began polling.
2. **Per-Request HTTP Client Allocation Overhead**: Inbound polling, chat detection, message dispatch, and callback handling created an ad-hoc `reqwest::Client` on every network invocation. Each call incurred an unpooled TCP 3-way handshake and TLS 1.3 cryptographic negotiation against `api.telegram.org:443`, adding 300–600ms of latency per request.
3. **Head-of-Line Blocking in Inbound Dispatch**: Telegram updates were processed sequentially inside the single polling loop. Complex commands (e.g. running CLI inspections, capturing screenshots, or preparing file exports) blocked subsequent updates and delayed update offset acknowledgments (`getUpdates?offset=N`).

This specification establishes an event-driven, pooled, and non-blocking architecture that slashes end-to-end turnaround latency to **< 1.5 seconds** (typical 400–800ms roundtrip) while strictly preserving error visibility (`AppError`), thread safety, and First-In-First-Out (FIFO) ordering invariants.

---

## 2. Problem Statement & Latency Root Cause Breakdown

### 2.1 The 60-Second Disabled/Idle Sleep Trap

In `src-tauri/src/modules/telegram_inbound/daemon.rs`:
```rust
loop {
    let is_enabled_flag = {
        let config = load_config().unwrap_or_default();
        config.is_enabled && !config.bot_token.trim().is_empty()
    };

    if !is_enabled_flag {
        {
            let mut st = LAST_TELEGRAM_STATUS.write().await;
            st.is_running = false;
        }
        tokio::time::sleep(Duration::from_secs(60)).await; // <-- 60-SECOND BLOCKADE!
        continue;
    }
    // ...
    if clean_token.is_empty() {
        tokio::time::sleep(Duration::from_secs(30)).await; // <-- 30-SECOND BLOCKADE!
        continue;
    }
}
```

When a user in the UI configured a bot token and toggled "Enable Telegram Remote Control", `save_telegram_config` was invoked:
```rust
#[tauri::command]
pub async fn save_telegram_config(config: TelegramConfig) -> AppResult<()> {
    let is_enabled = config.is_enabled;
    telegram_inbound::save_config(&config)?;

    if is_enabled {
        telegram_inbound::start_telegram_daemon();
    } else {
        telegram_inbound::stop_telegram_daemon();
    }
    Ok(())
}
```

However, `start_telegram_daemon()` used an atomic gate:
```rust
let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
if is_already_running {
    return; // Thread is ALREADY running and trapped in 60s sleep!
}
```
Because the daemon thread was already active and trapped inside `tokio::time::sleep(Duration::from_secs(60))`, `start_telegram_daemon` exited immediately without waking the sleeping loop. Users experienced a seemingly broken UI where the bot took up to a minute to respond to `/start` or `/ping`.

### 2.2 Unpooled HTTP Client Creation Overhead

Across `daemon.rs`, `formatting.rs`, `detection.rs`, and `config.rs`, code repeatedly constructed ephemeral HTTP clients:
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(15))
    .build()
    .map_err(|e| AppError::Network(e.to_string(), None))?;
```

Each fresh `reqwest::Client` allocates a new internal connection pool, DNS resolver cache, and TLS session cache. Because `api.telegram.org` resides behind Cloudflare Anycast:
- TCP Handshake: 1 RTT (~80–180ms depending on geography).
- TLS 1.3 Key Exchange: 1 RTT (~80–180ms).
- HTTP Request/Response Transfer: 1 RTT (~80–180ms).
- Total penalty per message or action: **300–600ms**.

For multi-message bursts or immediate "typing" indicators followed by replies, this overhead compounded to > 1.2 seconds of network connection overhead alone.

### 2.3 Head-of-Line Blocking in Sequential Inbound Updates

The previous inbound loop processed updates in serial:
```rust
if let Some(updates) = body["result"].as_array() {
    for update in updates {
        // ...
        send_chat_action(&client, &clean_token, chat_id, "typing").await;
        let reply = deliver_ui_or_command(&clean_token, chat_id, text_raw, &saved_media_opt).await;
        // deliver_ui_or_command can take 2-4 seconds for screenshots or CLI execution!
    }
}
```

If Update #1 requested `/shot` (capturing OS desktop screenshot, encoding PNG, and uploading to CDN), Update #2 (e.g. `/ping` or a rapid button tap) remained blocked until Update #1 completely finished. Update offset acknowledgment was also delayed, risking duplicate delivery if the daemon restarted mid-execution.

### 2.4 End-to-End Latency Budget Comparison

| Metric / Stage | Legacy Architecture | Optimized Architecture | Delta / Gain |
| :--- | :--- | :--- | :--- |
| **Config Save to Daemon Wakeup** | 0 – 60,000ms (avg 30s) | < 5ms (`TELEGRAM_DAEMON_NOTIFIER`) | **~30,000ms faster** |
| **TCP / TLS Handshake** | 300 – 600ms per request | 0ms (Reused keep-alive socket) | **300–600ms saved** |
| **"Typing" Indicator Dispatch** | 450 – 800ms | 80 – 160ms | **350–640ms faster** |
| **Command Execution Concurrency** | Serial blocking (Head-of-Line) | Concurrent (Spawned Tokio task) | **Non-blocking (0ms wait)** |
| **Offset Acknowledgment** | Delayed until all commands finish | Immediate upon batch intake | **Immediate offset safety** |
| **Total Roundtrip Turnaround** | **35,000 – 65,000ms** (worst case)<br/>**2,500 – 5,000ms** (idle case) | **< 1,500ms** (typical 400–800ms) | **Sub-2s Real-Time Guarantee** |

---

## 3. Target Architecture & Design Principles

```mermaid
flowchart TD
    subgraph UI_And_Commands [Frontend & Tauri IPC Commands]
        UI[Telegram Settings UI] -->|save_telegram_config| CMD_SAVE[commands/telegram.rs]
        CLI[AGM CLI / Daemon] -->|start_telegram_daemon| CMD_START[commands/telegram.rs]
        CMD_SAVE -->|notify_telegram_daemon| NOTIFIER[TELEGRAM_DAEMON_NOTIFIER (tokio::sync::Notify)]
    end

    subgraph Daemon_Core [telegram_inbound/daemon.rs]
        DAEMON[start_telegram_daemon Loop]
        NOTIFIER -.->|Instant Wakeup <5ms| DAEMON
        DAEMON -->|Long-Poll getUpdates 25s| POOL_CLIENT[TELEGRAM_HTTP_CLIENT (reqwest::Client)]
        POOL_CLIENT -->|api.telegram.org| TG_API[Telegram Bot API]
        TG_API -->|Batch Updates [U1, U2, ...]| DAEMON
        DAEMON -->|Immediate Advance Offset| OFFSET[last_update_id = max(id)]
    end

    subgraph Concurrent_Execution [Asynchronous Worker Spawning]
        DAEMON -->|spawn worker task| T1[Task 1: Callback Query]
        DAEMON -->|spawn worker task| T2[Task 2: Standard Command]
        T1 -->|answerCallbackQuery <100ms| POOL_CLIENT
        T1 -->|deliver_ui_or_command| EXEC1[Action Pipeline]
        T2 -->|sendChatAction 'typing' <150ms| POOL_CLIENT
        T2 -->|deliver_ui_or_command| EXEC2[Action Pipeline]
    end
```

### 3.1 Reactive Telegram Daemon Wakeup Architecture

To eliminate the 60s/30s sleep traps, the daemon transitions from static duration sleeps to reactive event barriers powered by `tokio::sync::Notify`:

```rust
use once_cell::sync::Lazy;
use tokio::sync::Notify;

/// Global event notifier to awaken the Telegram daemon immediately from sleep traps
pub static TELEGRAM_DAEMON_NOTIFIER: Lazy<Notify> = Lazy::new(Notify::new);

/// Signal the Telegram daemon to wake up immediately (e.g. after config save or enable toggle)
pub fn notify_telegram_daemon() {
    TELEGRAM_DAEMON_NOTIFIER.notify_one();
}
```

In the background daemon loop, every idle or disabled sleep is wrapped in `tokio::select!`:
```rust
if !is_enabled_flag {
    {
        let mut st = LAST_TELEGRAM_STATUS.write().await;
        st.is_running = false;
    }
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(60)) => {},
        _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
            tracing::info!("[TelegramDaemon] Reactive wakeup received while disabled; checking configuration immediately");
        }
    }
    continue;
}
```

Whenever `save_telegram_config` is called, `notify_telegram_daemon()` is triggered. If the daemon was in the 60s disabled sleep or 30s missing token sleep, the loop awakens in **< 5ms**, re-reads the updated configuration from disk, and begins polling immediately.

### 3.2 Persistent Pooled Telegram HTTP Client

All Telegram networking components share a single, globally initialized HTTP client configured for aggressive keep-alive, TCP connection reuse, and bounded connection pool timeouts:

```rust
/// Global pooled HTTP client for Telegram Bot API operations
pub static TELEGRAM_HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(35))
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("[TelegramClient] Failed to build pooled client with custom options: {e}; falling back to default");
            reqwest::Client::new()
        })
});
```

#### Key Settings:
- **`pool_idle_timeout(90s)`**: Long-polling iterations (25s) fall comfortably within the 90s idle window, ensuring the TCP connection never closes between poll cycles.
- **`tcp_keepalive(30s)`**: Periodically verifies network reachability through NAT gateways and corporate firewalls.
- **`timeout(35s)`**: Accommodates 25s server-side long-polling wait plus 10s TCP transmission buffer.
- **`pool_max_idle_per_host(10)`**: Allows simultaneous background message sends, callback answers, and long-polling requests without socket starvation.

### 3.3 Non-blocking Inbound Processing Model

In the previous design, `start_telegram_daemon` evaluated `deliver_ui_or_command` synchronously inside the iteration loop. In the optimized architecture:
1. When a batch of updates arrives from `getUpdates`, the daemon updates `last_update_id` to `max(update_id)` immediately.
2. Authorization and chat binding validation occur synchronously in memory.
3. Once validated, each update is dispatched into a detached Tokio task via `tauri::async_runtime::spawn`:
   - Task immediately fires `send_chat_action("typing")` or `answer_callback_query` using `TELEGRAM_HTTP_CLIENT`.
   - Task executes `deliver_ui_or_command` asynchronously.
   - Reply messages are chunked and transmitted without holding back the daemon's next long-polling cycle.
4. The main daemon loop yields minimally (50ms) and initiates the next `getUpdates?offset=N` call without waiting for command execution to complete.

---

## 4. Component Design & Structural Contracts

### 4.1 Module Structure & Exports (`src-tauri/src/modules/telegram_inbound/mod.rs`)

`mod.rs` centralizes all submodule declarations and public exports:
```rust
pub mod cluster;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod detection;
pub mod formatting;
pub mod injection;
pub mod media;
pub mod node_label;
pub mod open_shot;
pub mod palette;
pub mod processing;
pub mod prompt_history;
pub mod reports_a;
pub mod reports_b;
pub mod reports_cluster_a;
pub mod reports_cluster_b;
pub mod send_flow;
pub mod send_listing;
pub mod tests;
pub mod types;
pub mod ui_remote;

pub use cluster::*;
pub use commands::*;
pub use config::*;
pub use daemon::*;
pub use detection::*;
pub use formatting::*;
pub use injection::*;
pub use media::*;
pub use node_label::*;
pub use open_shot::*;
pub use palette::*;
pub use processing::*;
pub use reports_a::*;
pub use reports_b::*;
pub use reports_cluster_a::*;
pub use reports_cluster_b::*;
pub use send_flow::*;
pub use tests::*;
pub use types::*;
```

### 4.2 Daemon Subsystem (`src-tauri/src/modules/telegram_inbound/daemon.rs`)

#### Contract 1: `TELEGRAM_DAEMON_NOTIFIER` & `notify_telegram_daemon`
```rust
pub static TELEGRAM_DAEMON_NOTIFIER: Lazy<tokio::sync::Notify> = Lazy::new(tokio::sync::Notify::new);

pub fn notify_telegram_daemon() {
    TELEGRAM_DAEMON_NOTIFIER.notify_one();
}
```

#### Contract 2: `TELEGRAM_HTTP_CLIENT`
```rust
pub static TELEGRAM_HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(35))
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("[telegram_inbound] Failed to initialize pooled HTTP client: {e}");
            reqwest::Client::new()
        })
});
```

#### Contract 3: Loop Structure with Reactive Wakeup
```rust
pub fn start_telegram_daemon() {
    let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        // If already running, notify to break any pending sleep immediately
        notify_telegram_daemon();
        return;
    }

    tauri::async_runtime::spawn(async move {
        let mut last_update_id = 0i64;

        loop {
            // Check running flag
            if !TELEGRAM_RUNNING.load(Ordering::SeqCst) {
                break;
            }

            let (is_enabled, clean_token) = {
                let config = load_config().unwrap_or_default();
                (config.is_enabled && !config.bot_token.trim().is_empty(), config.bot_token.trim().to_string())
            };

            if !is_enabled || clean_token.is_empty() {
                {
                    let mut st = LAST_TELEGRAM_STATUS.write().await;
                    st.is_running = false;
                }
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {},
                    _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
                        tracing::debug!("[TelegramDaemon] Reactive wakeup from idle/disabled sleep");
                    }
                }
                continue;
            }

            {
                let mut st = LAST_TELEGRAM_STATUS.write().await;
                st.is_running = true;
                st.last_poll_at = Utc::now().timestamp();
                st.error_message = None;
            }

            // Execute long-poll getUpdates using TELEGRAM_HTTP_CLIENT
            // ...
        }
    });
}
```

### 4.3 Command Handlers (`src-tauri/src/commands/telegram.rs`)

`save_telegram_config` must notify the daemon immediately on both enable and disable branches:
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

## 5. Error Handling & Invariant Enforcement

### 5.1 No-Silent-Errors Compliance
In accordance with the repository's house rule (2026-10-10):
- All network or configuration failures that prevent operation MUST return or log through `AppError` (`src-tauri/src/error.rs`).
- Best-effort auxiliary operations (e.g. typing indicators, button callback spinner dismissal, and chat ID persistence) MUST use `crate::error::record_ignored(...)` accompanied by an explicit `// Justification:` comment detailing why continuing is safe.

Example:
```rust
// Justification: best-effort callback query answer to dismiss button spinners immediately;
// failure logged without changing control flow or dropping user interaction
crate::error::record_ignored(
    TELEGRAM_HTTP_CLIENT
        .post(&ans_url)
        .json(&json!({ "callback_query_id": clean_cb }))
        .send()
        .await,
    "answerCallbackQuery",
);
```

### 5.2 FIFO Ordering & Monotonicity Invariants
All prompt restoration and queue dispatch operations triggered via Telegram (`/send`, `/prompt`, `/restore`) MUST preserve strict FIFO ordering:
- `ORDER BY created_at ASC, id ASC` across SQLite queries in `active_prompts`, `prompt_backups`, and `repo_prompts.db`.
- Offset monotonicity in `getUpdates`: `offset = max_update_id + 1` is strictly enforced and committed.

### 5.3 Thread Safety & Concurrency
- `TELEGRAM_RUNNING`: `Arc<AtomicBool>` with `Ordering::SeqCst` for deterministic state transitions.
- `LAST_TELEGRAM_STATUS`: `Arc<RwLock<TelegramWatcherStatus>>` avoiding deadlocks by strictly minimizing critical section durations.
- `TELEGRAM_DAEMON_NOTIFIER`: `tokio::sync::Notify` guarantees that calling `notify_one()` is completely non-blocking and safe from any thread or async task context.

---

## 6. Verification & Quality Assurance Strategy

### 6.1 Verification Scenarios & Latency Metrics

| Test Case | Scenario / Trigger | Expected Outcome | Verification Method |
| :--- | :--- | :--- | :--- |
| **TC-01** | Save Config when Disabled | Daemon wakes up within < 10ms; status transitions to `is_running: true` without waiting 60s. | Telemetry audit of `last_poll_at` vs save timestamp. |
| **TC-02** | HTTP Client Connection Pooling | Multiple sequential requests reuse identical TCP 4-tuple without renegotiating TLS. | Verification of socket reuse and sub-200ms roundtrips. |
| **TC-03** | Rapid Inbound Burst | Two updates (e.g. `/ping` and `/status`) sent simultaneously are executed concurrently without head-of-line blocking. | Background tasks spawned simultaneously; both replies delivered in < 1.5s. |
| **TC-04** | Toggle Disable | Calling `save_telegram_config` with `is_enabled: false` halts the daemon immediately and sets `is_running: false`. | `get_telegram_status` returns `is_running: false` within 100ms. |

### 6.2 Pre-flight Checks (Cargo & Linters)
- `cd src-tauri && cargo fmt -- --check`
- `cd src-tauri && cargo clippy --all-targets --all-features`
- Strict relative Git paths invariant: verified zero absolute paths in documentation, code comments, and manifests.

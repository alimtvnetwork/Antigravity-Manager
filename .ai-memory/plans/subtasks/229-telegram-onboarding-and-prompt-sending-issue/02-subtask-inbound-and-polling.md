---
plan: 229-telegram-onboarding-and-prompt-sending-issue
subtask: "02"
title: Daemon Inbound Polling Loop Refactor, Instant Prompt Queue Awakening & Webhook Self-Healing
domain: backend-rust
depends_on: "01"
citations:
  architecture_spec: ../../../../02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/02-component-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/telegram_inbound/types.rs
  - src-tauri/src/modules/telegram_inbound/daemon.rs
  - src-tauri/src/modules/telegram_inbound/injection.rs
  - src-tauri/src/modules/telegram_inbound/config.rs
  - src-tauri/src/modules/telegram_inbound/detection.rs
  - src-tauri/src/commands/telegram.rs
status: pending
---

# Subtask 02: Daemon Inbound Polling Loop Refactor, Instant Prompt Queue Awakening & Webhook Self-Healing

## 1. Context & Motivation

Antigravity Manager integrates with the Telegram Bot API (`src-tauri/src/modules/telegram_inbound/`) for remote inspection, status reporting, and prompt injection. In production, four critical bottlenecks severely hurt developer velocity and onboarding reliability:

1. **60s / 30s Polling Sleep Traps in `daemon.rs`**: When the bot daemon runs while disabled or unconfigured, it sleeps for 60 seconds (`Duration::from_secs(60)`) or 30 seconds (`Duration::from_secs(30)`). Re-enabling the bot or saving a token in the UI takes up to a full minute before the daemon awakens and starts polling.
2. **Synchronous Execution Pipeline in `daemon.rs`**: Incoming commands (`/status`, `/instances`, `/nodes`, `/run`, document CSV/JSON exports, CDN image uploads) are awaited sequentially inline inside the update loop. Long-running SSH commands or export generations block offset updates and prevent responsive handling of concurrent messages.
3. **Delayed Prompt Dispatch (up to 30s idle gap) in `injection.rs`**: When an inbound prompt is saved to SQLite (`repo_db::save_or_requeue_prompt`), it sits waiting for the background `PromptQueueScheduler` (`src-tauri/src/modules/scheduler.rs`), which idles for 30 seconds (`PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL`) when the queue was previously empty.
4. **HTTP 409 Conflict Webhook Traps in `config.rs` & `detection.rs`**: If a bot token was previously associated with an external webhook service, Telegram Bot API returns `HTTP 409 Conflict`, causing auto-detection and long-polling to fail permanently.

This subtask provides the concrete implementation plan to refactor the polling loop with `tokio::sync::Notify`, detach heavy command execution into background tasks, trigger instant prompt queue awakening on injection, and implement auto-healing for webhook conflicts.

---

## 2. Target Files & Symbol Breakdown

| Target File | Changes / Symbols |
| :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/types.rs` | 1. Define `pub static TELEGRAM_DAEMON_NOTIFIER: Lazy<tokio::sync::Notify>`.<br/>2. Implement `pub fn notify_telegram_daemon()`. |
| `src-tauri/src/modules/telegram_inbound/daemon.rs` | 1. Replace 60s/30s uninterruptible sleeps in `start_telegram_daemon()` with `tokio::select!` on `TELEGRAM_DAEMON_NOTIFIER.notified()`.<br/>2. Spawning heavy command handlers (`deliver_ui_or_command`) into isolated Tokio tasks (`tauri::async_runtime::spawn`).<br/>3. Advance and acknowledge update offsets immediately to preserve Telegram polling monotonicity.<br/>4. Catch HTTP 409 Conflict in polling loop and trigger webhook deletion auto-heal. |
| `src-tauri/src/modules/telegram_inbound/injection.rs` | 1. In `execute_prompt_injection()`, immediately after `repo_db::save_or_requeue_prompt(&active_prompt)`, trigger `crate::modules::scheduler::notify_prompt_queue()`.<br/>2. Spawn blocking dispatch probe `crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&resolved_instance))` to launch immediately without waiting for scheduler ticks. |
| `src-tauri/src/modules/telegram_inbound/config.rs` | 1. Implement `pub async fn delete_telegram_webhook(bot_token: &str) -> Result<bool, AppError>`.<br/>2. Detach `register_telegram_bot_commands` in `test_telegram_connection` into a background task to drop latency from 3s to < 200ms. |
| `src-tauri/src/modules/telegram_inbound/detection.rs` | Intercept HTTP 409 Conflict during `getUpdates` in `detect_telegram_chat_id`, auto-invoke `delete_telegram_webhook`, and retry once. |
| `src-tauri/src/commands/telegram.rs` | In `save_telegram_config()`, invoke `telegram_inbound::notify_telegram_daemon()` on configuration save to wake the daemon immediately. |

---

## 3. Step-by-Step Implementation Protocol

### 3.1 Step 1: Add `TELEGRAM_DAEMON_NOTIFIER` in `types.rs`
1. In `src-tauri/src/modules/telegram_inbound/types.rs`, declare:
   ```rust
   pub static TELEGRAM_DAEMON_NOTIFIER: once_cell::sync::Lazy<tokio::sync::Notify> =
       once_cell::sync::Lazy::new(tokio::sync::Notify::new);

   pub fn notify_telegram_daemon() {
       TELEGRAM_DAEMON_NOTIFIER.notify_one();
   }
   ```
2. Ensure both symbols are re-exported in `src-tauri/src/modules/telegram_inbound/mod.rs`.

---

### 3.2 Step 2: Implement Webhook Self-Healing in `config.rs`
1. In `src-tauri/src/modules/telegram_inbound/config.rs`, implement `delete_telegram_webhook`:
   ```rust
   /// Permanently delete any active webhook configuration on Telegram Bot API
   /// to allow getUpdates long-polling to function without HTTP 409 Conflict.
   pub async fn delete_telegram_webhook(bot_token: &str) -> Result<bool, AppError> {
       let clean_token = bot_token.trim();
       if clean_token.is_empty() {
           return Err(AppError::Config("Bot token cannot be empty".to_string()));
       }

       let url = format!(
           "https://api.telegram.org/bot{}/deleteWebhook?drop_pending_updates=false",
           clean_token
       );
       let client = reqwest::Client::builder()
           .timeout(Duration::from_secs(10))
           .build()
           .map_err(|e| AppError::Network(e.to_string(), None))?;

       let resp = client
           .post(&url)
           .send()
           .await
           .map_err(|e| AppError::Network(e.to_string(), None))?;

       let status = resp.status().as_u16();
       if status != 200 {
           return Err(AppError::Network(
               format!("deleteWebhook failed with HTTP status {}", status),
               Some(status),
           ));
       }

       let body: Value = resp
           .json()
           .await
           .map_err(|e| AppError::Network(e.to_string(), None))?;

       let is_ok = body["ok"].as_bool().unwrap_or(false);
       if is_ok {
           tracing::info!("[TelegramConfig] Successfully deleted active Telegram webhook");
       } else {
           let desc = body["description"].as_str().unwrap_or("Unknown error");
           tracing::warn!("[TelegramConfig] deleteWebhook returned non-ok: {}", desc);
       }
       Ok(is_ok)
   }
   ```
2. Refactor `test_telegram_connection` in `config.rs` to detach `register_telegram_bot_commands`:
   ```rust
   // Detach slash command registration to background Tokio task so test returns in < 200ms
   let token_clone = clean_token.clone();
   tauri::async_runtime::spawn(async move {
       // Justification: best-effort bot commands registration; failures logged without blocking token test
       crate::error::record_ignored(
           register_telegram_bot_commands(&token_clone).await,
           "register_telegram_bot_commands background",
       );
   });
   ```

---

### 3.3 Step 3: Auto-Heal 409 in `detection.rs`
1. In `src-tauri/src/modules/telegram_inbound/detection.rs`, modify `detect_telegram_chat_id`:
   ```rust
   let mut resp = client
       .get(&url)
       .send()
       .await
       .map_err(|e| AppError::Network(e.to_string(), None))?;

   // If 409 Conflict is returned, an active webhook is blocking getUpdates.
   // Automatically delete the webhook and retry once.
   if resp.status().as_u16() == 409 {
       tracing::warn!("[TelegramDetect] HTTP 409 Conflict detected on getUpdates; attempting delete_telegram_webhook auto-heal");
       if let Err(wh_err) = delete_telegram_webhook(clean_token).await {
           tracing::error!("[TelegramDetect] Failed to auto-delete webhook: {}", wh_err);
       } else {
           resp = client
               .get(&url)
               .send()
               .await
               .map_err(|e| AppError::Network(e.to_string(), None))?;
       }
   }
   ```

---

### 3.4 Step 4: Refactor Inbound Polling Loop in `daemon.rs`
1. In `start_telegram_daemon()`:
   - Replace `tokio::time::sleep(Duration::from_secs(60)).await;` with:
     ```rust
     tokio::select! {
         _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
             tracing::info!("[TelegramDaemon] Reactive wakeup: bot configuration enabled or updated");
         }
         _ = tokio::time::sleep(Duration::from_secs(60)) => {}
     }
     ```
   - Replace `tokio::time::sleep(Duration::from_secs(30)).await;` with:
     ```rust
     tokio::select! {
         _ = TELEGRAM_DAEMON_NOTIFIER.notified() => {
             tracing::info!("[TelegramDaemon] Reactive wakeup: bot token provided");
         }
         _ = tokio::time::sleep(Duration::from_secs(30)) => {}
     }
     ```
   - Add HTTP 409 Conflict handling on `getUpdates` response:
     ```rust
     if resp.status().as_u16() == 409 {
         tracing::warn!("[TelegramDaemon] HTTP 409 Conflict in polling loop. Auto-healing webhook...");
         let _ = delete_telegram_webhook(&clean_token).await;
         tokio::time::sleep(Duration::from_millis(500)).await;
         continue;
     }
     ```
   - Detach callback query handling:
     ```rust
     let client_clone = http_client.clone();
     let token_clone = clean_token.clone();
     let cb_id_clone = cb_id.clone();
     tauri::async_runtime::spawn(async move {
         answer_callback_query(&client_clone, &token_clone, &cb_id_clone).await;
     });
     ```
   - Detach command execution in `deliver_ui_or_command`:
     ```rust
     let token_clone = clean_token.clone();
     tauri::async_runtime::spawn(async move {
         let cmd_str = normalize_cmd_callback_payload(&data);
         let delivered = deliver_ui_or_command(&token_clone, chat_id, &cmd_str, &None).await;
         if delivered.is_none() && data.starts_with("cmd:") {
             crate::error::record_ignored(
                 send_telegram_message_with_markup(
                     &token_clone,
                     chat_id,
                     "That button is no longer valid — send a fresh command to get new buttons.",
                     None,
                     false,
                 ).await,
                 "send unknown callback action notice",
             );
         }
     });
     ```
   - Detach standard message execution and photo handling:
     ```rust
     let token_clone = config.bot_token.clone();
     tauri::async_runtime::spawn(async move {
         let saved_media_opt = download_and_store_telegram_photo(&token_clone, &msg).await;
         let _ = deliver_ui_or_command(&token_clone, chat_id, &text_raw, &saved_media_opt).await;
     });
     ```

---

### 3.5 Step 5: Instant Prompt Queue Awakening in `injection.rs`
1. In `src-tauri/src/modules/telegram_inbound/injection.rs`:
   - Locate lines 265–285 in `execute_prompt_injection`.
   - Right after `repo_db::save_or_requeue_prompt(&active_prompt)` succeeds:
     ```rust
     if let Err(e) = repo_db::save_or_requeue_prompt(&active_prompt) {
         return format!(
             "Couldn't save your prompt: {}",
             clean_for_telegram_html(&e, 200)
         );
     }

     // 1. Wake the adaptive prompt queue scheduler immediately
     crate::modules::scheduler::notify_prompt_queue();

     // 2. Launch an immediate dispatch probe for this instance
     let target_inst_for_dispatch = resolved_instance.clone();
     tauri::async_runtime::spawn_blocking(move || {
         match crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&target_inst_for_dispatch)) {
             Ok(count) => {
                 if count > 0 {
                     tracing::info!(
                         "[TelegramInjection] Immediate dispatch probe launched {} prompt(s) for instance '{}'",
                         count,
                         target_inst_for_dispatch
                     );
                 }
             }
             Err(err) => {
                 tracing::warn!(
                     "[TelegramInjection] Immediate dispatch probe error for instance '{}': {}",
                     target_inst_for_dispatch,
                     err
                 );
             }
         }
     });

     // 3. Attempt direct agy spawn if not already launched
     let spawned = match repo_db::spawn_prompt_via_agy(&active_prompt) {
         Ok(()) => true,
         Err(spawn_err) => {
             crate::modules::logger::log_warn(&format!(
                 "[Telegram] agy spawn failed for inbound prompt: {spawn_err}"
             ));
             false
         }
     };
     ```

---

### 3.6 Step 6: Hook Configuration Awakening in `commands/telegram.rs`
1. In `src-tauri/src/commands/telegram.rs`:
   - In `save_telegram_config(config: TelegramConfig) -> AppResult<()>`:
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
         // Wake up daemon loop immediately so config changes take effect without waiting
         telegram_inbound::notify_telegram_daemon();
         Ok(())
     }
     ```

---

## 4. Error Handling & Invariant Enforcement

1. **No Silent Errors Rule**:
   - Every fallible operation returning `Result` must propagate via `?` with `AppError` context, or use `crate::error::record_ignored` accompanied by an explicit `// Justification:` comment explaining why the error is non-fatal.
2. **Monotonic FIFO Query Invariant**:
   - `check_and_dispatch_enqueued_prompts` enforces strict FIFO ordering (`ORDER BY created_at ASC, id ASC`). No inverted `DESC` queries are allowed.
3. **Caller IDE Self-Preservation**:
   - Inbound prompt injections target workspace instances through standard `repo_db` and `agy` dispatch channels, never terminating active developer processes.

---

## 5. Rollback & Risk Mitigation Protocol

1. **Rollback Strategy**:
   - If detached background tasks cause out-of-order execution for commands from the same user, wrap per-chat command dispatch in a lightweight per-chat channel queue (`mpsc::Sender`).
   - If `delete_telegram_webhook` encounters network timeouts, it fails gracefully without altering saved configuration or blocking manual chat ID entry.
2. **Backward Compatibility**:
   - Existing configs (`telegram_config.json`) remain 100% compatible.
   - All existing Telegram slash commands (`/tree`, `/active`, `/nodes`, `/status`, `/prompt`, `/send`) preserve their exact interfaces and output formats.

---

## 6. Subtask Acceptance Criteria

- [ ] `TELEGRAM_DAEMON_NOTIFIER` and `notify_telegram_daemon()` are defined in `types.rs` and called on `save_telegram_config`.
- [ ] In `daemon.rs`, all 60s and 30s sleeps in `start_telegram_daemon` are governed by `tokio::select!` with `TELEGRAM_DAEMON_NOTIFIER.notified()`.
- [ ] Command execution (`deliver_ui_or_command`) and callback query answers in `daemon.rs` are dispatched into spawned background Tokio tasks.
- [ ] In `injection.rs`, saving a prompt immediately calls `crate::modules::scheduler::notify_prompt_queue()` and probes `repo_db::check_and_dispatch_enqueued_prompts(Some(&resolved_instance))`.
- [ ] `delete_telegram_webhook` is implemented in `config.rs` and called when HTTP 409 Conflict is detected in `detection.rs` and `daemon.rs`.
- [ ] `test_telegram_connection` detaches `register_telegram_bot_commands` to background execution and returns in $< 500\text{ms}$.
- [ ] Zero build commands or unit test executions were performed during spec authoring.
- [ ] All file paths are strictly repository-relative Git paths.

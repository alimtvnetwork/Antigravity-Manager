---
plan: 229-telegram-onboarding-and-prompt-sending-issue
subtask: "03"
title: Concurrent Channel Broadcasting, Detached Command Registration, and Seamless Frontend Onboarding Auto-Save
domain: fullstack-rust-typescript
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/01-architecture-spec.md
  migration_spec: ../../../../02-spec/21-app/229-telegram-onboarding-and-prompt-sending-issue/03-migration-and-verification-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/notification_hub/channels.rs
  - src-tauri/src/modules/telegram_inbound/config.rs
  - src/pages/Telegram.tsx
  - src/components/settings/EmailNotifications/useTelegramHandlers.ts
status: pending
---

# Subtask 03: Concurrent Channel Broadcasting, Detached Command Registration, and Seamless Frontend Onboarding Auto-Save

## 1. Context & Motivation

In the current Antigravity Manager codebase, several latency bottlenecks and usability friction points impede the operator experience during notification delivery and Telegram setup:

1. **Sequential Switch Broadcasting Latency**:
   In `src-tauri/src/modules/notification_hub/channels.rs`, `deliver_switch_channels` sequentially executes Email, Telegram, and Supabase alerts. Because `dispatch_email_switch_alert` is a synchronous SMTP socket operation that can block for 1 to 5 seconds, Telegram alerts and Supabase fleet synchronizations are unnecessarily delayed.
2. **Blocking Command Registration in Bot Verification**:
   In `src-tauri/src/modules/telegram_inbound/config.rs`, `test_telegram_connection` calls `getMe` to verify the bot token, but then synchronously awaits `register_telegram_bot_commands(clean_token).await` (`setMyCommands`) before returning. This adds 800ms–2000ms of extra round-trip network latency to a simple token validation check.
3. **Double-Step Onboarding Friction**:
   In both `src/pages/Telegram.tsx` and `src/components/settings/EmailNotifications/useTelegramHandlers.ts`, clicking "Auto-Detect Chat ID" populates the detected chat ID into React state, but does not automatically set `is_enabled = true` or invoke `telegramService.saveConfig(...)`. Users must manually remember to click "Save Configuration" or risk losing the detected ID upon navigation.

This subtask resolves these issues with surgical backend concurrency and seamless frontend auto-persistence.

---

## 2. Target Files & Symbols Breakdown

| Target File | Changes / Operations |
| :--- | :--- |
| `src-tauri/src/modules/notification_hub/channels.rs` | Refactor `deliver_switch_channels`: Offload synchronous `dispatch_email_switch_alert` into `tokio::task::spawn_blocking` and run concurrent fan-out across Email, Telegram (`dispatch_telegram_switch_alert`), and Supabase (`push_and_read_instance_email`) via `tokio::join!`. |
| `src-tauri/src/modules/telegram_inbound/config.rs` | Refactor `test_telegram_connection`: Return immediately after `getMe` verification (<200ms) and dispatch `register_telegram_bot_commands` as a detached `tokio::spawn` background task with proper error recording. |
| `src/pages/Telegram.tsx` | Update `handleDetectChat`: Automatically set `is_enabled = true` and call `telegramService.saveConfig(updatedConfig)` upon successful chat ID detection, refreshing watcher status and notifying the user. |
| `src/components/settings/EmailNotifications/useTelegramHandlers.ts` | Update `handleDetectTelegramChatId`: Automatically set `is_enabled = true` and persist via `telegramService.saveConfig(updatedConfig)` upon successful chat ID detection. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Concurrent Broadcast Fan-Out in `channels.rs`

1. Open `src-tauri/src/modules/notification_hub/channels.rs`.
2. Locate `async fn deliver_switch_channels(details: &SwitchNotificationDetails) -> String`.
3. Restructure the delivery logic:
   - Perform local memory broadcast: `dispatch_self_json_in_use_broadcast(details);`.
   - Wrap the blocking email dispatch in `tokio::task::spawn_blocking`:
     ```rust
     let email_details = details.clone();
     let email_task = tokio::task::spawn_blocking(move || {
         record_channel(
             NotificationChannelType::Email,
             dispatch_email_switch_alert(&email_details),
         )
     });
     ```
   - Encapsulate the Telegram alert in an async future:
     ```rust
     let telegram_fut = async {
         record_channel(
             NotificationChannelType::Telegram,
             dispatch_telegram_switch_alert(details).await,
         )
     };
     ```
   - Encapsulate Supabase fleet sync in an async future:
     ```rust
     let supabase_fut = async {
         record_channel(
             NotificationChannelType::Supabase,
             crate::modules::supabase_sync::push_and_read_instance_email(&details.instance_id)
                 .await
                 .map_err(|err| redact_secrets(&err.to_string())),
         )
     };
     ```
   - Execute all three concurrently using `tokio::join!`:
     ```rust
     let (email_res, telegram, supabase) = tokio::join!(
         email_task,
         telegram_fut,
         supabase_fut,
     );
     let email = email_res.unwrap_or_else(|join_err| {
         let err_msg = format!("email task panicked or cancelled: {}", join_err);
         crate::modules::logger::log_error(&format!("[Notify] email: FAIL {}", err_msg));
         format!("[Notify] email: FAIL {}", err_msg)
     });
     format!("{}\n{}\n{}", email, telegram, supabase)
     ```
4. Verify that secret redaction via `redact_secrets()` is strictly preserved on all output and error paths.

---

### 3.2 Step 2: Detached Bot Command Registration in `config.rs`

1. Open `src-tauri/src/modules/telegram_inbound/config.rs`.
2. Locate `pub async fn test_telegram_connection(bot_token: &str) -> Result<String, AppError>`.
3. In `test_telegram_connection`, reduce HTTP timeout to 5 seconds.
4. Replace the sequential await of `register_telegram_bot_commands`:
   ```rust
   // Previously:
   // crate::error::record_ignored(
   //     register_telegram_bot_commands(clean_token).await,
   //     "register_telegram_bot_commands",
   // );

   // New Detached Implementation:
   let token_for_task = clean_token.to_string();
   tokio::spawn(async move {
       // Justification: best-effort command registration in background; failure tracked in error log without blocking test_telegram_connection
       crate::error::record_ignored(
           register_telegram_bot_commands(&token_for_task).await,
           "register_telegram_bot_commands",
       );
   });

   Ok(username)
   ```
5. Ensure compliance with the "No Silent Errors" house rule: `crate::error::record_ignored` must have a valid justification comment.

---

### 3.3 Step 3: Frontend Auto-Enable & Auto-Save in `src/pages/Telegram.tsx`

1. Open `src/pages/Telegram.tsx`.
2. Locate `const handleDetectChat = async () => { ... }`.
3. Update the success branch after `telegramService.detectChatId`:
   - Construct `updatedConfig` with `allowed_chat_id: detected.chat_id` and `is_enabled: true`.
   - Update React state via `setConfig(updatedConfig)`.
   - Await `telegramService.saveConfig(updatedConfig)` immediately.
   - Fetch updated daemon status: `const newStat = await telegramService.getStatus(); setStatus(newStat);`.
   - Display success toast: `showToast(\`Chat ID \${detected.chat_id} detected and saved! Listener activated.\`, 'success');`.
   - Wrap the auto-save call in a nested `try/catch`, routing failures to `useErrorStore.getState().trackWarning` with a fallback toast advising the user to click Save manually.

---

### 3.4 Step 4: Frontend Auto-Enable & Auto-Save in `useTelegramHandlers.ts`

1. Open `src/components/settings/EmailNotifications/useTelegramHandlers.ts`.
2. Locate `const handleDetectTelegramChatId = async () => { ... }`.
3. Update the success branch after `telegramService.detectChatId`:
   - Construct `updatedConfig` with `allowed_chat_id: res.chat_id` and `is_enabled: true`.
   - Update state via `setTelegramConfig(updatedConfig)`.
   - Await `telegramService.saveConfig(updatedConfig)`.
   - Fetch updated status: `const st = await telegramService.getStatus(); setTelegramStatus(st);`.
   - Display confirmation toast.
   - Catch auto-save errors via `useErrorStore.getState().trackWarning`.

---

## 4. Resilience & Fallback Guarantees

1. **SMTP Failure Isolation**:
   If the SMTP server times out, refuses connections, or panics, the join error is caught by `unwrap_or_else`. The Telegram and Supabase alerts are guaranteed to complete and dispatch without being cancelled or slowed.
2. **Background Command Registration Resilience**:
   If `setMyCommands` fails due to Telegram rate limiting or network glitches, the error is safely recorded via `record_ignored`. The user's token verification succeeds instantaneously (<200ms) based on the genuine `getMe` response.
3. **Graceful Auto-Save Recovery**:
   If disk I/O fails during configuration auto-save, the detected chat ID remains active in the UI form state, and a warning is logged to `useErrorStore` while prompting the user to retry saving manually.

---

## 5. Coding Guidelines & Invariants Checklist

- [x] **Strictly Relative Paths**: All paths cited are relative to the repository root (e.g. `src-tauri/src/modules/notification_hub/channels.rs`).
- [x] **No Silent Errors Rule**:
  - Rust: Background task uses `crate::error::record_ignored` with explicit `// Justification:` comment.
  - TypeScript: Best-effort auto-save catches use `useErrorStore.getState().trackWarning(...)`.
- [x] **Positive Booleans**: Uses `is_enabled = true` (never inverted or negative booleans).
- [x] **Secret Sanitization**: All logged strings and error messages pass through `redact_secrets()`.
- [x] **Zero Build/Test Execution During Spec Phase**: Spec authored without invoking premature compilers.

---

## 6. Acceptance Criteria & Verification Protocol

| ID | Verification Scenario | Target Outcome |
| :--- | :--- | :--- |
| **AC-01** | Concurrent Broadcast Fan-Out | When account switch occurs with a slow or unresponsive SMTP configuration, Telegram alert is delivered to the operator's phone in `< 400ms`. |
| **AC-02** | Token Test Latency (<200ms) | Clicking "Verify Token" in the GUI returns the bot username in under 200ms without waiting for slash commands to be set. |
| **AC-03** | Auto-Detect & Auto-Save Flow | Clicking "Auto-Detect Chat ID" in `Telegram.tsx` detects the chat ID, toggles "Enable Listener" to active, writes `telegram_config.json`, and starts the polling daemon without requiring a secondary click on "Save Configuration". |
| **AC-04** | Settings Modal Parity | Auto-detect in `EmailNotifications` modal (`useTelegramHandlers.ts`) triggers the same auto-save and status update. |
| **AC-05** | Compiler & Linter Verification | `cargo clippy --all-targets --all-features` and `cargo fmt -- --check` pass with zero warnings; `npm run build` succeeds cleanly. |

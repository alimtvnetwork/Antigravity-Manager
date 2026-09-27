# 53 — Telegram Bot Auto-Connect, Chat ID Discovery & Full Remote Command Suite

## 1. Objective & Scope

This specification defines the end-to-end Telegram Bot integration for Antigravity-Manager across the GUI (`Email & Alerts` and `Supabase Sync` settings panels), the native terminal CLI (`agm telegram`), and the background inbound polling daemon (`telegram_inbound.rs`).

> **Security Invariant:** Bot tokens (`<BOT_ID>:<SECRET>`) and personal/group numeric `chat_id` values must NEVER be committed to Git, source files, specifications, or README documentation. Runtime credentials reside strictly in the local user profile (`~/.antigravity_tools/telegram_config.json`).

---

## 2. Chat ID Auto-Discovery & Zero-Friction Connection

### 2.1 How Telegram Chat ID Discovery Works
Telegram bots cannot initiate messages to a user until the user first sends a message (e.g., `/start` or `/ping`) to the bot (`t.me/<bot_username>`). Once the user sends any message to the bot:
1. Antigravity-Manager queries `GET https://api.telegram.org/bot<TOKEN>/getUpdates`.
2. The response array contains `message.chat.id` (a numeric integer such as `123456789` for private chats or `-100123456789` for supergroups/channels), along with `message.chat.username` and `message.chat.first_name`.
3. `detect_telegram_chat_id(bot_token)` extracts the most recent `chat.id` from `getUpdates`.
4. If `getUpdates` is empty and Telegram Desktop is running locally, AGM can optionally open the deep link `tg://resolve?domain=<bot_username>&text=/ping` to pre-fill `/ping` in the bot chat window.
5. Additionally, when the background polling daemon (`start_telegram_daemon`) is active and `allowed_chat_id` is `None`, the first incoming message automatically binds and persists `allowed_chat_id = Some(chat_id)`.

### 2.2 Bot Slash-Command Menu Registration
Upon connecting or testing a bot token, AGM automatically invokes `POST https://api.telegram.org/bot<TOKEN>/setMyCommands` to register the interactive `/` command menu in Telegram clients:
- `/ping` — Verify node connectivity, IP, Git version & uptime
- `/status` — Full node, account quota & proxy status
- `/observe` — Inspect live workspaces & running prompt queues
- `/gitmap` — Run GitMap CLI command (e.g. `/gitmap pe`)
- `/agm` — Run AGM CLI command (e.g. `/agm accounts`, `/agm wpr`)
- `/api` — Query API proxy health & active account bindings
- `/backup` — Backup running prompts to split SQLite DB (`/backup` or `/backpack`)
- `/restore` — Restore backed-up prompts to resume execution
- `/email` — Check email status or send test/help email (`/email status`, `/email help`, `/email ping`)
- `/ff` — Fast-forward switch to highest-quota standby account
- `/snapshot` — View multi-node cluster status snapshot
- `/help` — Show full Telegram remote command reference

---

## 3. UI Integration (`Settings -> Email & Alerts` & `Settings -> Supabase Sync`)

The **Telegram Bot Remote & Alerts** configuration panel is accessible directly in **Settings -> Email & Alerts** (`EmailNotificationSettings.tsx`) as well as **Settings -> Supabase Sync** (`SupabaseSyncSettings.tsx`):
- **Telegram Bot Token**: Password/text input for the token from `@BotFather`.
- **Allowed Chat ID**: Numeric input with an adjacent **"Auto-Detect Chat ID"** button (`detect_telegram_chat_id` Tauri command) that queries `getUpdates` and populates the Chat ID automatically.
- **Open in Telegram**: Deep-link launcher (`tg://resolve?domain=<bot_username>&text=/ping`) once the bot username is verified via `Test Bot`.
- **Action Buttons**:
  - **Test Bot**: Calls `getMe`, displays `@<bot_username>`, registers slash commands via `setMyCommands`, and auto-detects Chat ID if empty.
  - **Auto-Detect Chat ID**: Queries `getUpdates` and fills `allowed_chat_id`.
  - **Send Ping**: Sends an immediate telemetry ping card to `allowed_chat_id`.
  - **Save Telegram**: Persists configuration to `~/.antigravity_tools/telegram_config.json` and activates the background polling daemon.

---

## 4. Terminal CLI Interface (`agm telegram`)

The `agm telegram` CLI provides full headless parity:
- `agm telegram connect <bot_token> [chat_id]` (or `agm telegram set <bot_token> [chat_id]`):
  - Validates the token via `getMe`.
  - If `[chat_id]` is omitted, automatically queries `getUpdates` via `detect_telegram_chat_id` to discover the user's numeric Chat ID.
  - Saves `telegram_config.json` with `is_enabled = true`, registers the bot command menu via `setMyCommands`, and dispatches a live welcome message to the chat.
- `agm telegram detect-chat-id [bot_token]` (alias `chat-id`):
  - Queries `getUpdates` and prints discovered Chat IDs, usernames, and chat types, saving the latest Chat ID if none was configured.
- `agm telegram ls [--json]`:
  - Displays current Telegram configuration (with masked token in text mode), bot status, and configured Chat ID.
- `agm telegram ping`:
  - Sends a formatted telemetry ping message (Version, Commit Hash, Branch, Last Release, Node Alias, IP, Uptime) to the configured Telegram chat.
- `agm telegram observe` (alias `status`):
  - Collects live node telemetry, active account quota, running workspaces, and prompt queues, prints to terminal, and sends the formatted report to Telegram.
- `agm telegram send "<message>"` (alias `notify "<message>"`):
  - Sends a custom notification message to the configured Telegram chat.
- `agm telegram gitmap <args...>`:
  - Executes `gitmap <args...>` locally, prints output, and forwards the result to the Telegram chat.
- `agm telegram api`:
  - Queries local API proxy status and forwards the report to the Telegram chat.
- `agm telegram backup [ls|restore]` (alias `backpack`):
  - Backs up running prompts to `backup-prompts.db` (or lists/restores backups) and sends a confirmation receipt to Telegram.
- `agm telegram email [status|ping|help]`:
  - Executes email status/ping/help dispatch and sends the receipt to Telegram.
- `agm telegram poll [--once]` (alias `watch`):
  - Polls Telegram `getUpdates`, executes any pending inbound commands (`/ping`, `/status`, `/observe`, `/gitmap`, `/agm`, `/api`, `/backup`, `/restore`, `/email`, `/ff`, `/snapshot`, `/help`), and replies directly to the chat.

---

## 5. Acceptance Criteria

- **AC-TG-001 (Zero Credential Leakage):** No real bot token or user Chat ID appears in any tracked repository file.
- **AC-TG-002 (Chat ID Auto-Discovery):** Both UI (`Auto-Detect Chat ID`) and CLI (`agm telegram connect <token>`) discover the numeric `chat_id` from `getUpdates` without requiring third-party `@userinfobot` lookups.
- **AC-TG-003 (End-to-End Command Execution):** Inbound and CLI-triggered commands (`ping`, `observe`/`status`, `email`, `gitmap`, `api`, `backup`/`backpack`, `restore`, `send`/`notify`) execute cleanly and deliver HTML-formatted responses to Telegram.

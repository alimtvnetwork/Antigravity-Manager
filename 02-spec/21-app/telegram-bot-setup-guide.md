# Telegram Bot Setup, Auto-Discovery & Remote Command Guide

This guide explains how to connect a Telegram Bot to **Antigravity Manager (AGM)** using either the Desktop UI or the `agm telegram` CLI terminal, how to automatically discover and bind your `allowed_chat_id` without third-party bots, and how to run remote telemetry, GitMap, API, Email, and Backup ("Backpack") commands from Telegram.

> **Security Notice**: Never commit your real Telegram Bot HTTP API token or numeric Chat ID to Git, README files, or shared repositories. AGM stores your credentials strictly in your local machine profile at `~/.antigravity_tools/telegram_config.json`.

---

## 1. Create a Telegram Bot via `@BotFather`

1. Open Telegram and start a chat with [`@BotFather`](https://t.me/BotFather).
2. Send `/newbot` and follow the prompts:
   - **Bot Display Name**: e.g. `My AGM Node Bot`
   - **Bot Username**: Must end in `bot` (e.g. `my_agm_node_bot`)
3. `@BotFather` will return an **HTTP API Token** in the format:
   ```text
   123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ
   ```
4. Click the `t.me/<your_bot_username>` link in BotFather's message, click **Start**, and send `/ping` to your bot.

---

## 2. How to Find & Auto-Detect Your Telegram `Allowed Chat ID`

You **do not** need to look up your Chat ID manually using third-party bots. Antigravity Manager provides **three built-in ways** to discover and bind your `allowed_chat_id` directly from Telegram's `getUpdates` queue:

### Method A: One-Click UI Auto-Detection
1. Open **Settings → Email & Alerts** (scroll to **Telegram Bot & Alert Notifications**) or **Settings → Supabase Sync** (scroll to **Telegram Bot Integration**).
2. Paste your **Bot Token** into the **Telegram Bot Token** field.
3. Open your bot in Telegram and send `/ping` (or `/start`).
4. Click **Auto-Detect Chat ID** (or **Auto-Detect from `/ping`**) in the UI.
5. AGM queries `getUpdates`, automatically fills the **Allowed Chat ID (Numeric)** field, verifies the `@bot_username`, and enables the daemon.
6. Click **Save Telegram Settings** and **Send Test Alert**.

### Method B: Zero-Touch First-Message Auto-Binding
1. Paste only your **Bot Token** in the UI, check **Enable Telegram Bot Notifications & Inbound Daemon**, leave **Allowed Chat ID** blank, and click **Save Telegram Settings**.
2. Send `/ping` to your bot in Telegram.
3. The background polling daemon automatically captures your `chat.id` from the first message, locks `allowed_chat_id` to your personal chat in `~/.antigravity_tools/telegram_config.json`, registers the bot's slash-command menu, and replies with a confirmation!

### Method C: One-Command Terminal CLI (`agm telegram connect`)
Run a single command in your terminal after sending `/ping` to your bot:
```bash
agm telegram connect "<YOUR_BOT_TOKEN>"
```
Or if you already know your numeric Chat ID:
```bash
agm telegram connect "<YOUR_BOT_TOKEN>" <YOUR_CHAT_ID>
```
What `agm telegram connect` does automatically:
- Verifies the token against `getMe`
- Auto-detects `allowed_chat_id` from `getUpdates` if omitted
- Saves `~/.antigravity_tools/telegram_config.json` with `is_enabled: true`
- Registers the slash-command menu (`setMyCommands`) on your bot
- Dispatches a live node & Git telemetry report directly to your Telegram chat

---

## 3. Terminal CLI Reference (`agm telegram`)

You can control and test every Telegram integration feature directly from any terminal (PowerShell, CMD, Bash, Zsh):

| CLI Command | Description |
| :--- | :--- |
| `agm telegram connect <TOKEN> [CHAT_ID]` | Verify token, auto-detect Chat ID (if omitted), save config, register slash commands, and send a welcome ping |
| `agm telegram detect-chat-id [TOKEN]` | Auto-detect numeric Chat ID from recent `/start` or `/ping` messages sent to the bot |
| `agm telegram ls [--json]` | Show current Telegram bot status, masked token, bound Chat ID, and polling interval |
| `agm telegram ping` | Send a live node health & Git telemetry report (`version`, `commit`, `branch`, `last_tag`, `IP`) to Telegram |
| `agm telegram observe` | Send a full node observation report (active account, model quota, running vs idle workspaces, prompt queue, backups) |
| `agm telegram email [status\|ping\|help]` | Send email vault status to Telegram, or trigger an outbound SMTP `ping` / `help` email report |
| `agm telegram gitmap [args]` | Execute `gitmap <args>` (default `pe`) and forward the live CI/CD pipeline matrix to Telegram |
| `agm telegram api` | Send API proxy & account pool status (`agm status` + `agm accounts`) to Telegram |
| `agm telegram backup [ls]` | Trigger a split-SQLite prompt backup (`backup_active_prompts_to_split_db`) or list backup batches (`ls`) and notify Telegram |
| `agm telegram restore` | Restore the latest prompt backup batch and send confirmation to Telegram |
| `agm telegram send "<MESSAGE>"` | Send any custom notification message directly to your bound Telegram chat |
| `agm telegram poll [--once]` | Poll and execute inbound `/commands` from Telegram (`--once` processes pending updates and exits) |
| `agm telegram enable` / `disable` | Enable or disable the background Telegram polling daemon |
| `agm telegram clear` | Remove local Telegram bot credentials |

---

## 4. Interactive Bot Slash Commands (Inside Telegram Chat)

Once connected, you can send any of the following commands directly to your bot in Telegram:

| Bot Slash Command | Action Executed on Host Node |
| :--- | :--- |
| `/ping` | Returns instant `PONG` with node hostname, local IP, AGM version, Git commit hash, branch, and last release tag |
| `/observe` or `/status` | Comprehensive observation report: active account, minimum model quota %, live running vs idle workspaces, prompt queue, and backup count |
| `/gitmap pe` | Runs `gitmap pe` on the host machine and replies with the live GitHub Actions CI/CD status matrix |
| `/gitmap <subcommand>` | Runs safe GitMap commands (`pe`, `status`, `version`, `doctor`, `prompts ls`) and returns output |
| `/agm status` or `/api` | Queries active account, proxy gateway status, and account pool summary |
| `/agm <subcommand>` | Runs `agm status`, `agm accounts`, `agm prompts ls`, or `agm version` |
| `/backup` or `/backpack` | Captures all active/running workspace prompts into encrypted split SQLite (`backup_prompts.db`) |
| `/backup ls` or `/backpack ls` | Lists the most recent split SQLite backup batches |
| `/restore` | Restores and re-queues the latest backed-up prompt batch |
| `/email status` | Reports configured SMTP/IMAP mailboxes, recipients, and IMAP watcher daemon state |
| `/email ping` | Dispatches a full HTML status & telemetry email to all active recipients |
| `/email help` | Dispatches the full HTML command manual & cheat-sheet email to all active recipients |
| `/ff` | Fast-forwards (rotates) to the next highest-quota account and auto-resumes active project prompts |
| `/snapshot` | Captures a live desktop screenshot of the host machine and uploads it to your Telegram chat |
| `/help` | Displays the interactive command manual inside Telegram |

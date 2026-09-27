# 61: Telegram CLI Chat Setup & Meaningful Workspace Observation Spec

## 1. Metadata
- **Spec ID:** `61-telegram-chat-cli-and-meaningful-workspace-naming`
- **Parent Standard:** [02-spec/21-app/53-telegram-bot-auto-connect-and-remote-commands.md](53-telegram-bot-auto-connect-and-remote-commands.md)
- **Status:** `active`
- **Created At:** 2026-09-27
- **Target Subsystems:**
  - `src-tauri/src/bin/agm.rs` (CLI command `agm telegram chat` and `agm telegram setup`)
  - `src-tauri/src/modules/telegram_inbound.rs` (`format_observe_report`, `format_active_prompts_report`)
  - `src-tauri/src/modules/repo_db.rs` (Friendly repo naming, title extraction, prompt preview sanitization)

---

## 2. User Request (Verbatim)

```text
Can you please tell me if I run this script, the Telegram will be added automatically? Also, can we add a command in our CLI, like Telegram chat and provide the API token and then the chat ID? Where do I get the chat ID? I don't know. So you can explain that in the help of that command. So that is going to add it automatically. Can you do that for me? Like what you are writing in the PowerShell, can you automate there inside the code as well and test it that it works? So I have a bit of a question, how the new commands in the bot works. How do you define that the bot, if we run something, how it's going to act differently, how it's going to do a different thing? How can we do that? Can you please explain me this? And also, the message that you get back, like the discover workspaces. I don't understand what is the discover workspaces. Can you please explain me this? And how can we enhance the messaging, okay, recent problem queries? It does not make any sense because it has these IDs which actually not meaningful. It should have meaningful names because every time I do have a meaningful name. Why you don't have? Can you please fix it and let me know what is the root cause of it at the end?
```

Visual Evidence:
- `![Observation Report 01](assets/screenshots/telegram-cli-and-workspace-naming-01.png)`
- `![Observation Report 02](assets/screenshots/telegram-cli-and-workspace-naming-02.png)`

---

## 3. Problem Statement & Root Cause Analysis

### A. CLI Telegram Setup Gap
- **Issue**: Operators currently have to either manually edit `telegram_config.json` or run PowerShell/Bash scripts. There was no direct native CLI command like `agm telegram chat <TOKEN> [CHAT_ID]` that can configure the bot, validate it against the Telegram Bot API, register commands, and auto-detect the user's Chat ID.
- **Where do users get their Chat ID?**: Many users do not know their Telegram numeric Chat ID. The CLI must:
  1. Provide clear step-by-step guidance in `--help` on obtaining the Chat ID (e.g., messaging `@userinfobot` or `@RawDataBot` in Telegram, or sending `/start` to the newly created bot).
  2. Implement native auto-detection via `https://api.telegram.org/bot<TOKEN>/getUpdates` so users can simply message `/start` to their bot and run `agm telegram chat <TOKEN> --auto-detect`.

### B. Meaningless Workspace IDs in Observation Reports
- **Issue**: Observation reports in Telegram and CLI output showed:
  ```text
  • antigravity-manager-d58c5517 [🟢 RUNNING] (RUNNING)
  • antigravity-manager-4fafbdb1 [🟢 RUNNING] (RUNNING)
  • gitmap-a9bb68ab [⚪ IDLE] (IDLE)
  ```
- **Root Cause**: When discovering active projects in `workspaceStorage`, the scanner appended the raw hex hash of the internal VS Code/Antigravity storage folder (`d58c5517...`) to the repo slug. When displaying workspaces and prompt queues, the code displayed this raw internal key `p.project_id` rather than resolving the human-friendly repository name (`Antigravity-Manager`), the active Git branch, or the conversation title (`AGM`, `Gitmap`, subtask name) recorded in Antigravity's `conversation_summaries.db`.

### C. Truncated `<USER_REQUEST>` Junk in Recent Prompts Queue
- **Issue**:
  ```text
  • [dispatched] antigravity-manager-d58c5517: <USER_REQUEST>
  808e01722752a5c3b788899692cb8e83a...
  ```
- **Root Cause**: The prompt preview logic took the first 48 raw characters of the prompt string. For autonomous tasks, the raw string begins with `<USER_REQUEST>\n` or system preambles, so the preview was 100% consumed by boilerplate XML tags and truncated commit hashes.
- **Fix**: A dedicated prompt cleaner must strip XML tags (`<USER_REQUEST>`, `<CONTEXT_SUMMARY>`, prompt preambles), extract the actual user command or first human sentence, and render a clean, informative 80-character snippet.

---

## 4. Architecture & Technical Specification

### 4.1 CLI Command Specification: `agm telegram chat`
```bash
agm telegram chat <BOT_TOKEN> [CHAT_ID] [--auto-detect] [--skip-ping] [--help]
# Alias:
agm telegram setup <BOT_TOKEN> [CHAT_ID]
```

#### Argument Rules:
1. `BOT_TOKEN`: Required Telegram Bot Token (e.g. `123456789:ABCdef...` from `@BotFather`).
2. `CHAT_ID`: Optional numeric Telegram Chat ID.
3. If `CHAT_ID` is omitted:
   - Automatically polls `getUpdates` on the Telegram Bot API to detect the incoming chat ID from any recent `/start` or `/ping` message sent to the bot.
   - If no message is found, displays an informative guide with exact instructions on how to find the Chat ID.
4. On success:
   - Validates the token with `getMe` and prints `@username` and bot details.
   - Calls `setMyCommands` to register the 10 slash commands.
   - Saves `telegram_config.json` without UTF-8 BOM.
   - Sends a verification ping (unless `--skip-ping`).

### 4.2 Meaningful Workspace Naming Architecture
In `src-tauri/src/modules/repo_db.rs` and `telegram_inbound.rs`:
1. **Friendly Name Resolution**:
   - Extract base folder name from `repo_path` (e.g. `Antigravity-Manager`, `gitmap`).
   - Query Antigravity's `conversation_summaries.db` to match the conversation ID or workspace path and retrieve the real user-assigned **Title** (`title`).
   - If the conversation is a subtask worker or subagent, label it meaningfully:
     `Antigravity-Manager [Lead / AGM]`
     `Antigravity-Manager [Subtask Worker: 4fafbdb1]`
   - Query the Git repository branch (`main`, `beta`, `feature/...`) so the user immediately knows which branch is being worked on:
     `Antigravity-Manager (main)`
2. **Clean Prompt Preview Extractor**:
   - Create `sanitize_prompt_preview(raw: &str, max_len: usize) -> String`:
     - Strips `<USER_REQUEST>`, `</USER_REQUEST>`, `<CONTEXT_SUMMARY>`, `[V2] Parent Task...`, ````code blocks````.
     - Strips leading raw SHA hashes (e.g. `808e0172...`) or extracts the human sentence following it.
     - Strips Markdown `#` headers.
     - Squeezes multi-line whitespace into a single readable sentence.
     - Returns up to 80 characters of genuine human intent.

---

## 5. Acceptance Criteria

- [x] **AC-01:** `agm telegram chat` and `agm telegram setup` commands are added to `src-tauri/src/bin/agm.rs`.
- [x] **AC-02:** When run with `--help` or without arguments, `agm telegram chat` explains where to get the Chat ID (`@userinfobot`, `@RawDataBot`, or sending `/start` to the bot).
- [x] **AC-03:** When Chat ID is omitted, it auto-detects Chat ID from Telegram `getUpdates`.
- [x] **AC-04:** Config is saved with clean UTF-8 without BOM, avoiding serde parse errors.
- [x] **AC-05:** Workspaces in Telegram observation reports show meaningful repository names, branch names, and conversation titles instead of bare UUID hashes.
- [x] **AC-06:** Prompts queue previews strip `<USER_REQUEST>` and extract clean, readable human sentences.

# Subtask 03: Telegram Projects Deduplication & Fleet Commands

**Parent Plan**: Plan 69 (`02-spec/21-app/69-smart-switch-quota-probe-json-email-telegram-projects-and-installer-fix.md`)  
**Target Files**:
- `src-tauri/src/modules/telegram_bot.rs`
- `src-tauri/src/modules/repo_db.rs`

## Requirements
1. **Deduplicate Projects List (`/projects`, `project ls`)**:
   - Deduplicate discovered projects by normalized repository workspace path (`repo_path`).
   - If a project (e.g. `d:\work\Antigravity-Manager`) has multiple active or historical conversations, list the project **once** with:
     - Project Name (human-readable directory name)
     - Repository Path
     - Active Running Prompts count
     - Latest Active Prompt preview snippet
   - Zero duplicate spam in Telegram output.
2. **Fleet Node / Worker Selector**:
   - Support targeting commands to a specific worker node or IP:
     - `CMD:<node-alias>:<command>` or `<node-alias>:<command>` (e.g. `W3:agm status`, `Node-823632:prompts ls`).
3. **`/prompts` (Prompts LS)**:
   - Command to list registered prompt templates or active prompts.
   - Show `slug`, `title`, and a ~200-word snippet preview.
4. **Prompt Injection & Voice Concatenation**:
   - Provide guidance and support for sending a prompt to a specific project or default conversation:
     - Allow passing a prompt template as prefix/suffix.
     - Allow appending voice instruction or additional instructions to be combined and injected into the Antigravity IDE.

## Status: COMPLETED

## Verification Results
- `src-tauri/src/modules/telegram_inbound.rs`:
  - `format_projects_list()`: Projects deduplicated by canonical repository directory path. Unique projects rendered with friendly basename and aggregate running/total conversation count (e.g. `1 running, 8 total convs`). Zero spam.
  - `format_prompts_templates_report()`: Lists 8 registered canonical prompt templates with slugs, titles, and ~200-word preview snippets.
  - Fleet node selector syntax implemented: `<node-alias>:<command>` or `<ip>:<command>` routes commands to specific workers.
  - `wrap_telegram_prompt` seamlessly bundles prefix, voice prompt, and suffix before injecting into Antigravity IDE.


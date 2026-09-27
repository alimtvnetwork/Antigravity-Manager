# RCA-22: Cryptic Workspace UUIDs and Truncated `<USER_REQUEST>` Previews in Telegram Reports

## 1. Issue Metadata
- **Issue ID:** `22-meaningless-workspace-uuid-and-truncated-prompt-preview-rca`
- **Application:** Antigravity-Manager (AGM)
- **Subsystems:** `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/telegram_inbound.rs`
- **Severity:** Medium (UX / Telemetry Degradation)
- **Status:** Resolved

---

## 2. Reproduction & Observation

In Telegram `/status` and `/observe` reports:
```text
📂 Discovered Workspaces:
• antigravity-manager-d58c5517 [🟢 RUNNING] (RUNNING)
• antigravity-manager-4fafbdb1 [🟢 RUNNING] (RUNNING)
• gitmap-a9bb68ab [⚪ IDLE] (IDLE)

📝 Recent Prompts Queue:
• [dispatched] antigravity-manager-d58c5517: <USER_REQUEST>
808e01722752a5c3b788899692cb8e83a...
```

---

## 3. Root Cause Analysis (4-Part)

### 3.1 Why did it happen?
1. **Workspace ID Leakage**: In `repo_db::detect_running_projects`, the directory scanner for VS Code / Antigravity `workspaceStorage` took `entry.file_name()` (which is a random 8-character hex hash representing the internal IDE workspace state) and concatenated it:
   ```rust
   let project_id = format!("{}-{}", repo_name.to_lowercase(), entry.file_name().to_string_lossy());
   ```
   When recording active prompts and formatting reports, this raw internal hash `project_id` was passed directly to the user-facing Telegram formatter.
2. **Conversation Titles Discarded**: Antigravity's internal SQLite database `conversation_summaries.db` contains human-assigned and AI-generated conversation **Titles** (`title`), e.g., `"AGM"`, `"Gitmap"`, but the loader assigned it to `_title` and discarded it.
3. **Naive Slicing of Raw Prompt Strings**: `format_observe_report()` called `clean_for_telegram_html(&p.prompt_content, 48)`. Because autonomous agent requests start with `<USER_REQUEST>\n` followed by git hashes or system preambles, the first 48 characters contained zero human information.

---

## 4. Remediation & Code Fix

1. **Title & Branch Enrichment**:
   - In `repo_db.rs`, store and resolve the real workspace title from `conversation_summaries.db` and active Git branch from `git rev-parse --abbrev-ref HEAD`.
   - Render clean workspace labels:
     - `Antigravity-Manager (main) [Lead]`
     - `Antigravity-Manager [Worker: 4fafbdb1]`
     - `Gitmap (main)`
2. **Smart Prompt Sanitizer**:
   - Implement `sanitize_prompt_preview()` to strip XML tags (`<USER_REQUEST>`, `<CONTEXT_SUMMARY>`), leading 40-character SHAs, and markdown headers, and extract the first actual English sentence / user command.
   - Expand preview length from 48 to 80 characters for readability.
3. **Native CLI Command**:
   - Add `agm telegram chat <TOKEN> [CHAT_ID]` (and `agm telegram setup`) with auto-detection of Chat ID from Telegram `getUpdates` and instructions on `@userinfobot`.

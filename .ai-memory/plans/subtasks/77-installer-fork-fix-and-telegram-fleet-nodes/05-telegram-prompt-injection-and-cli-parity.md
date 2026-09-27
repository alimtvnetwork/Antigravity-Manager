# Subtask 05: Telegram Prompt Injection & CLI Parity

## Metadata
- **Parent Plan:** `77-installer-fork-fix-and-telegram-fleet-nodes.md`
- **Status:** Completed
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound.rs`
  - `src-tauri/src/bin/agm.rs`

---

## 1. Description
Enable remote prompt injection from Telegram and mirror all capabilities to the `agm` CLI:
1. Parse `/prompt <node> <project> <text>` or `/prompt <project> <text>`.
2. For local target, dispatch to `repo_db` and trigger workspace execution.
3. For remote target, enqueue prompt into Supabase Secondary DB `command_queue`.
4. Add CLI commands in `src-tauri/src/bin/agm.rs`:
   - `agm telegram nodes`
   - `agm telegram prompts`
   - `agm telegram prompt <node> <project> <text>`

---

## 2. Verification Criteria
- [x] `cargo check --bin agm` passes without errors.
- [x] CLI command `agm telegram nodes` runs and prints cluster nodes.

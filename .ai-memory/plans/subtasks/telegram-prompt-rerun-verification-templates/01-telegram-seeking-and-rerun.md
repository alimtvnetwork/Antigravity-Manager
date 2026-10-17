# Subtask 01: Telegram Conversation Seeking, ≥200-Word Expandable Rerun & Verification

## Status
- **Status:** `pending`
- **Owner:** Worker 01
- **Phase:** Phase 2 (Implementation)
- **Specification:** `02-spec/21-app/telegram-prompt-rerun-verification-templates/01-telegram-seeking-and-rerun-spec.md`
- **Target Files:**
  - `src-tauri/src/modules/repo_db.rs`
  - `src-tauri/src/modules/telegram_inbound/mod.rs`
  - `src-tauri/src/modules/telegram_inbound/processing.rs`
  - `src-tauri/src/modules/telegram_inbound/reports_b.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/tests.rs`

---

## Deliverables & Implementation Checklist

### Step 1: Re-export Turn Extraction in `src-tauri/src/modules/repo_db.rs`
- [ ] In `src-tauri/src/modules/repo_db.rs`, re-export `extract_conversation_prompt_turns` from `transcript` module so `telegram_inbound` can access conversation prompt turns directly.
- [ ] Verify that `AgmPromptTurnNode` is already exported from `models`.

### Step 2: Implement Seeking & Expandable Turn Reporting in `src-tauri/src/modules/telegram_inbound/reports_b.rs`
- [ ] Add `pub async fn format_conversation_seek_report(query_str: &str) -> (String, Option<serde_json::Value>)`:
  - Enforce FIFO database queries (`ORDER BY created_at ASC, id ASC`) when loading prompt and conversation lists.
  - Calculate total running prompts and active instances across the workspace.
  - Filter conversation candidates by query term:
    * Sequence codes: `#C001`, `P001`, `GM:#1`
    * 8-char conversation ID: `cid8`
    * Project/Workspace name: e.g. `Antigravity-Manager`
    * Free-text search matching conversation title or prompt content.
  - Surface instance sequence token (`[AGM:P001 | GM:#1]`), conversation sequence (`#C001`), instance name, and clean conversation title.
  - For each conversation, extract prompt turns via `repo_db::extract_conversation_prompt_turns`.
  - Format the latest turn(s) with ≥200-word preview using `repo_db::extract_prompt_words_preview`.
  - Wrap multiline prompt previews in Telegram Bot API 7.2+ expandable blockquotes:
    ```html
    <blockquote expandable><pre><code class="language-markdown">{turn_preview}</code></pre></blockquote>
    ```
  - Ensure unclosed HTML tags are balanced across dispatches.
  - Attach an interactive `InlineKeyboardMarkup` row for the active/latest turn:
    * 🔁 Rerun Turn: `callback_data: "send:t:<cid8>:<turn>"`
    * 🛡️ Verify Turn: `callback_data: "send:v:<cid8>:<turn>"`
    * 📋 Copy Turn: `copy_text: { "text": clamped_prompt }` (Bot API 7.8+)

### Step 3: Command Registration in `src-tauri/src/modules/telegram_inbound/processing.rs`
- [ ] Register new command triggers in `process_telegram_command_text`:
  ```rust
  "seek" | "conversations" | "convs" | "turns" => {
      let (reply, markup) = super::reports_b::format_conversation_seek_report(rest).await;
      // Return formatted reply (markup handled by Telegram Bot message dispatcher)
      Some(reply)
  }
  ```
- [ ] Ensure `process_telegram_update` passes optional `reply_markup` through to Telegram Bot API `sendMessage`.

### Step 4: Documentation Update in Command Manual (`reports_b.rs`)
- [ ] Update `format_help_manual()` in `src-tauri/src/modules/telegram_inbound/reports_b.rs` under the **Fleet telemetry** or **Easy Machine & Instance Prompting** sections:
  - Add `• <code>/seek [query]</code> — Seek running conversations with ≥200-word expandable prompt previews &amp; rerun buttons`
  - Add `• <code>/conversations</code> — Alias for <code>/seek</code> conversation browser`
- [ ] Verify test `help_manual_substitutes_all_format_args` passes without placeholder mismatch.

### Step 5: Turn Callback Dispatching in `src-tauri/src/modules/telegram_inbound/send_flow.rs`
- [ ] In `handle_send_callback`, intercept `send:t:` and `send:v:` prefixes:
  ```rust
  if let Some(rest) = payload.strip_prefix("t:") {
      return handle_turn_rerun_callback(chat_id, rest, false).await;
  }
  if let Some(rest) = payload.strip_prefix("v:") {
      return handle_turn_rerun_callback(chat_id, rest, true).await;
  }
  ```
- [ ] Implement `handle_turn_rerun_callback(chat_id: i64, payload: &str, is_verify: bool) -> (String, Option<Value>)`:
  - Enforce RCA 15 60-byte callback limit check (`MAX_CALLBACK_BYTES = 60`).
  - Parse `<cid8>` and `<turn>`.
  - Lookup target conversation in `repo_db` and read prompt turns via `repo_db::extract_conversation_prompt_turns`.
  - Extract prompt text for specified turn.
  - If `is_verify` is true, prefix prompt with verification testing directive:
    `"Review, verify, and run regression tests for the following implementation:\n"`.
  - Dispatch prompt via `super::injection::execute_prompt_injection`.
  - Trigger `crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance`.
  - Capture proof screenshot via `super::open_shot::capture_and_send_proof_shot`.
  - Return formatted delivery receipt card with screenshot proof.

### Step 6: Unit Testing in `src-tauri/src/modules/telegram_inbound/tests.rs`
- [ ] `test_turn_callback_payload_bounds`: Assert `send:t:<cid8>:<turn>` and `send:v:<cid8>:<turn>` remain strictly <= 60 bytes.
- [ ] `test_expandable_blockquote_tag_balance`: Assert all formatted seek cards maintain balanced `<blockquote expandable>` and `</blockquote>` tags.
- [ ] `test_conversation_seek_filtering`: Test sequence code parsing (`#C001`, `P001`, `GM:#1`) and free-text matching.
- [ ] `test_fifo_ordering_invariant`: Verify all DB queries maintain `ORDER BY created_at ASC, id ASC`.

### Step 7: Pre-flight Verification Gate
- [ ] Run `cd src-tauri && cargo fmt -- --check`.
- [ ] Run `cd src-tauri && cargo clippy --all-targets --all-features`.

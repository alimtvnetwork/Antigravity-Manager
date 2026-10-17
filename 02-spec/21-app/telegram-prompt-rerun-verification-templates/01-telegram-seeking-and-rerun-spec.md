# Specification: Telegram Conversation Seeking, ≥200-Word Expandable Rerun & Verification

> **Target:** `02-spec/21-app/telegram-prompt-rerun-verification-templates/01-telegram-seeking-and-rerun-spec.md`  
> **Status:** APPROVED  
> **Slug:** `telegram-prompt-rerun-verification-templates`  
> **Scope:** Telegram inbound conversation discovery, prompt turn retrieval, expandable blockquotes, and inline rerun/verify keyboard actions (`src-tauri/src/modules/telegram_inbound/`, `src-tauri/src/modules/repo_db/`)

---

## User Request (Verbatim)

> "Also rerun option or seeking conversation, how many prompts are running, at least 200 words, and even expanding this option should be there from the Telegram, and I should be able to resend based on the conversation, just like the prompt tree view. We should have some sort of smart way of dealing with it in a smarter way to send it back to the Telegram. And I could pick which prompts I wanted to rerun or send for verification. So the option needs to be very smooth."

---

## Problem Classification & Motivation

While AGM provides rich desktop UI views (`PromptTreeSection.tsx`) and Telegram commands for high-level status (`/status`, `/active`, `/tree`, `/history`), operators interacting with Antigravity via Telegram previously faced several severe usability and visibility barriers:

1. **Truncated Visibility in Chat**: Existing Telegram prompt summaries (`/active`, `/query`) clamp text to short 100-word snippets or single lines, obscuring critical task instructions and multiline context.
2. **Missing Conversation-Centric Seeking**: Operators could not query conversations by sequence codes (`#C001`, `P001`, `GM:#1`), workspace names, or running status to inspect individual prompt turns with at least 200 words of verbatim context.
3. **Absence of Expandable Folding**: Long prompts sent to Telegram either polluted the screen with massive walls of text or required separate `/expand <id>` round-trips. Telegram Bot API 7.2 introduced `<blockquote expandable>` and Telegram Bot API 7.8 introduced inline `copy_text` buttons, but inbound command reporting did not exploit these capabilities.
4. **Disjointed Rerun & Verification Execution**: To rerun or verify a prompt from Telegram, users had to manually retype or copy-paste text between terminals. There was no one-tap turn picker that directly re-injected a specific turn into its originating conversation with optional verification wrapper directives, CDP composer typing, and proof-of-delivery screenshot capture.

---

## Architecture & System Invariants

```
                                  TELEGRAM CHAT
                                        │
           ┌────────────────────────────┴───────────────────────────┐
           ▼                                                        ▼
  /seek [query] or /conversations                   Inline Callbacks:
           │                                        • send:t:<cid8>:<turn> (Rerun)
           ▼                                        • send:v:<cid8>:<turn> (Verify)
 ┌────────────────────────────────────────┐         • copy_text (Bot API 7.8+)
 │ telegram_inbound/processing.rs         │                         │
 └───────────────────┬────────────────────┘                         ▼
                     ▼                             ┌─────────────────────────────────┐
 ┌────────────────────────────────────────┐        │ telegram_inbound/send_flow.rs   │
 │ telegram_inbound/reports_b.rs          │        └────────────────┬────────────────┘
 │ • format_conversation_seek_report()    │                         │
 │ • Running prompt counters              │                         ▼
 │ • [AGM:P001 | GM:#1] sequence tokens   │        ┌─────────────────────────────────┐
 │ • ≥200-word preview clamping           │        │ repo_db: extract turns by cid8  │
 │ • <blockquote expandable> formatting   │        └────────────────┬────────────────┘
 └───────────────────┬────────────────────┘                         │
                     ▼                                              ▼
 ┌────────────────────────────────────────┐        ┌─────────────────────────────────┐
 │ repo_db::extract_conversation_turns    │        │ Dispatch Pipeline:              │
 │ • transcript_full.jsonl precedence     │        │ 1. execute_prompt_injection     │
 │ • FIFO invariant (created_at ASC)      │        │ 2. submit_prompt_via_cdp        │
 └────────────────────────────────────────┘        │ 3. capture_and_send_proof_shot  │
                                                   └─────────────────────────────────┘
```

### Invariant 1: FIFO Ordering in Database Queries
All prompt and conversation queries across SQLite databases (`active_prompts`, `backup_prompts`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic and database queries; display sorting (newest first for human operator convenience) is performed strictly in-memory on loaded `Vec` structures.

### Invariant 2: 60-Byte Callback Payload Cap (RCA 15)
Telegram Bot API caps `callback_data` at 64 bytes (`BUTTON_DATA_INVALID`). AGM enforces `MAX_CALLBACK_BYTES = 60`. Turn callback payloads must adhere strictly to:
- Rerun Turn: `send:t:<cid8>:<turn>` (7 + 8 + 1 + ~3 = ~19 bytes, safely <= 60 bytes).
- Verify Turn: `send:v:<cid8>:<turn>` (7 + 8 + 1 + ~3 = ~19 bytes, safely <= 60 bytes).
Never embed raw prompt text, workspace paths, or full 36-character UUIDs into inline keyboard callback data.

### Invariant 3: Expandable Blockquote Balancing & HTML Hygiene
All multi-line prompt turn previews must be enclosed in Telegram Bot API 7.2+ expandable blockquotes:
```html
<blockquote expandable><pre><code class="language-markdown">{turn_preview}</code></pre></blockquote>
```
If message length exceeds Telegram's 4,096-character limit, chunking must ensure open HTML tags (`<blockquote>`, `<pre>`, `<code>`, `<b>`, `<i>`) are cleanly balanced and reopened across chunks to prevent `PARSE_ERROR` drops.

### Invariant 4: Dual Delivery & Proof of Execution
When a turn rerun or verification action is triggered:
1. Dispatch prompt text via canonical injection `telegram_inbound::injection::execute_prompt_injection`.
2. Attempt CDP direct submission into the live Antigravity composer via `modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance`.
3. Capture visual proof of delivery via `telegram_inbound::open_shot::capture_and_send_proof_shot`.
4. Report status back to Telegram with execution metrics and the captured proof screenshot.

---

## Detailed Inbound Telegram Specification

### 1. Inbound Commands (`processing.rs`)

Add command routing for `/seek` and `/conversations` (with aliases `/convs`, `/turns`):

```rust
"seek" | "conversations" | "convs" | "turns" => {
    Some(super::reports_b::format_conversation_seek_report(rest).await)
}
```

#### Command Semantics:
- `/seek`: Lists all currently running or active conversations across all instances. For each conversation, displays running prompt count, instance sequence token `[AGM:P001 | GM:#1]`, conversation sequence `#C001`, title, and the latest prompt turns (clamped to ≥200 words).
- `/seek <query>`: Filters conversations by:
  - Conversation Sequence Code: `#C001`, `C001`, `c1`
  - Project Sequence Token: `P001`, `GM:#1`
  - 8-char conversation ID: `d58c5517`
  - Workspace / Repo path: `Antigravity-Manager`, `gitmap`
  - Keyword search in prompt text or conversation title.

### 2. Report Formatting (`reports_b.rs`)

`reports_b.rs` exposes:
```rust
pub async fn format_conversation_seek_report(query_str: &str) -> (String, Option<serde_json::Value>)
```

#### Output Structure:
1. **Header Banner**:
   ```html
   🎯 <b>CONVERSATION SEEK &amp; PROMPT TURNS</b>
   ━━━━━━━━━━━━━━━━━━━━━━━━
   • <b>Running Prompts:</b> <code>{active_count}</code> active across <code>{inst_count}</code> instances
   • <b>Filter:</b> <code>{query_label}</code>
   ```
2. **Conversation Card**:
   ```html
   📂 <b>[AGM:P001 | GM:#1]</b> <code>#C001</code> — <b>{project_name}</b>
   • <b>Instance:</b> <code>{instance_id}</code> · <b>ID:</b> <code>{cid8}</code>
   • <b>Title:</b> <i>{conversation_title}</i>
   • <b>Status:</b> 🟢 RUNNING (Turn {active_turn}/{total_turns})

   <b>Turn #{turn_num}</b> [<i>{turn_status}</i>]:
   <blockquote expandable><pre><code class="language-markdown">{turn_preview_200w}</code></pre></blockquote>
   ```
3. **Interactive Inline Keyboard**:
   Each conversation turn is accompanied by an inline keyboard row:
   ```json
   {
     "inline_keyboard": [
       [
         { "text": "🔁 Rerun Turn #1", "callback_data": "send:t:d58c5517:1" },
         { "text": "🛡️ Verify Turn #1", "callback_data": "send:v:d58c5517:1" },
         { "text": "📋 Copy Turn", "copy_text": { "text": "{clamped_prompt_text}" } }
       ]
     ]
   }
   ```

### 3. Clamping & Text Formatting (`sequences.rs` / `text_utils.rs`)

- Use `crate::modules::repo_db::extract_prompt_words_preview(prompt_text, 200)` to preserve vertical spacing and paragraph linebreaks while clamping cleanly at 200 words followed by an ellipsis `...`.
- Pass text through `clean_for_telegram_html(text, 2800)` to escape `<`, `>`, `&` while preserving valid HTML syntax inside `<pre><code>`.

### 4. Turn Callback Handler (`send_flow.rs`)

Extend `handle_send_callback` in `src-tauri/src/modules/telegram_inbound/send_flow.rs`:

```rust
// Turn Rerun: send:t:<cid8>:<turn>
if let Some(rest) = payload.strip_prefix("t:") {
    return handle_turn_rerun_callback(chat_id, rest, false).await;
}
// Turn Verification: send:v:<cid8>:<turn>
if let Some(rest) = payload.strip_prefix("v:") {
    return handle_turn_rerun_callback(chat_id, rest, true).await;
}
```

#### Resolution Logic in `handle_turn_rerun_callback`:
1. Parse `<cid8>` and `<turn_number>`. Reject with `send_unknown()` if malformed.
2. Search matching conversation in `repo_db` via `cid8` prefix lookup across active instances and prompt sequences.
3. Call `repo_db::extract_conversation_prompt_turns(&base_dir, &full_cid, is_running)` to retrieve `AgmPromptTurnNode` entries.
4. Extract the targeted `turn_number`'s `prompt_text`.
5. If `is_verify` is `true`, wrap the prompt text with the verification directive prefix:
   ```text
   Review, verify, and run regression tests for the following implementation:
   {prompt_text}
   ```
6. Build injection argument: `{conversation_id} --instance {instance_id} {injected_text}`.
7. Execute `telegram_inbound::injection::execute_prompt_injection(&args).await`.
8. Concurrently or sequentially trigger `submit_prompt_via_cdp_for_instance(&instance_id, &injected_text)`.
9. Capture proof screenshot via `capture_and_send_proof_shot(&OpenShotTarget::Local).await`.
10. Return confirmation card detailing target machine, instance, sequence code, prompt word count, and delivery proof.

---

## Unit Testing & Verification Contract

1. **Callback Budget Test** (`test_turn_callback_fits_60_byte_cap`):
   Verify that `send:t:<cid8>:<turn>` and `send:v:<cid8>:<turn>` with 8-char UUIDs and 4-digit turn numbers never exceed 60 bytes.
2. **Expandable Blockquote Balancing Test** (`test_expandable_blockquote_tag_balance`):
   Verify that formatted seek cards contain exactly matching counts of `<blockquote expandable>`, `<pre>`, `<code>` and their closing tags `</code>`, `</pre>`, `</blockquote>`.
3. **Seek Report Query Parsing Test** (`test_seek_report_query_filtering`):
   Verify that query terms `#C001`, `P001`, `default`, and workspace substrings properly filter conversation candidates without throwing panics on empty or non-matching inputs.
4. **FIFO DB Invariant Verification**:
   Verify that all underlying SQLite lookups for conversation sequences and active prompts retain `ORDER BY created_at ASC, id ASC`.

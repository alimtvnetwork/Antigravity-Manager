# Subtask Plan 02: Backend Telegram Rerun Enqueue & FIFO Pipeline Alignment

- **Subtask Identifier:** `02-backend-rerun-enqueue`
- **Parent Master Plan:** `.ai-memory/plans/pending/227-rerun-and-attachment-format-correction.md`
- **Parent Component Spec:** `02-spec/21-app/227-rerun-and-attachment-format-correction/02-component-spec.md`
- **Assigned Worker:** Worker 02
- **Primary Attribution:** `@aukgit` (`(Thanks to @aukgit)`)
- **Status:** READY FOR EXECUTION
- **Disjoint Target Files:**
  - `src-tauri/src/modules/telegram_inbound/prompt_history.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`

---

## 1. Objectives & Context

Eliminate the immediate execution bypass and enforce the strict FIFO SQLite queue invariant for all Telegram prompt rerun operations:
1. Refactor `prompt_history.rs:rerun_prompt_by_id` to route through `crate::modules::repo_db::enqueue_prompt_for_instance` instead of `execute_prompt_injection`.
2. Ensure original image attachments (`image_payload`) are preserved and stored in `active_prompts` on rerun instead of being discarded.
3. Refactor `send_flow.rs:dispatch_turn_rerun_execution` to eliminate `execute_prompt_injection` and remove `submit_prompt_via_cdp_for_instance(..., true)`, routing through `enqueue_prompt_for_instance`.
4. Trigger reactive wakeup via `crate::modules::scheduler::notify_prompt_queue()` and best-effort idle dispatch check via `crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(instance_id))` across both entry points.
5. Adhere strictly to the No Silent Errors rule: all fallible operations must propagate errors or use `crate::error::record_ignored` with explicit `// Justification:` comments.

---

## 2. Step-by-Step Implementation Instructions

### Step 1: Pre-Execution Code Inspection
1. Inspect `src-tauri/src/modules/telegram_inbound/prompt_history.rs` around lines 95–215. Note how `prompt` is loaded from `list_all_prompts()`, how `prompt.instance_id`, `prompt.repo_path`, `prompt.prompt_content`, and `prompt.image_payload` are structured.
2. Inspect `src-tauri/src/modules/telegram_inbound/send_flow.rs` around lines 1144–1242. Note the arguments to `dispatch_turn_rerun_execution`: `cid8`, `full_cid`, `instance_id`, `repo_path`, `seq_code`, `gitmap_seq_code`, `turn_num`, `is_verify`, `prompt_content_opt`.

---

### Step 2: Refactoring `src-tauri/src/modules/telegram_inbound/prompt_history.rs`
In `rerun_prompt_by_id(id8: &str)`:
1. Locate lines 147–159 where `args` and `injected` are computed:
   ```rust
   let args = format!(
       "{conversation} --instance {instance} {}",
       prompt.prompt_content
   );
   let injected = super::injection::execute_prompt_injection(&args).await;
   ```
2. Replace `execute_prompt_injection` with the canonical queueing sequence:
   ```rust
   let mut enqueued = match crate::modules::repo_db::enqueue_prompt_for_instance(
       instance,
       &prompt.repo_path,
       &prompt.prompt_content,
       Some(&conversation),
   ) {
       Ok(p) => p,
       Err(err) => {
           return (
               format!(
                   "⚠️ <b>Failed to enqueue rerun:</b> <code>{}</code>",
                   clean_for_telegram_html(&err, 200)
               ),
               None,
           );
       }
   };

   // Preserve original image payload if present
   if let Some(ref img) = prompt.image_payload {
       let clean_img = img.trim();
       if !clean_img.is_empty() {
           enqueued.image_payload = Some(clean_img.to_string());
           // Justification: Persisting preserved image payload to SQLite queue is best-effort cache sync
           crate::error::record_ignored(
               crate::modules::repo_db::save_or_requeue_prompt(&enqueued),
               "persist preserved image payload for rerun prompt",
           );
       }
   }

   // Wake background scheduler immediately
   crate::modules::scheduler::notify_prompt_queue();

   // Immediate dispatch check if the instance is currently idle
   // Justification: Idle dispatch check is best-effort; background scheduler picks up prompt if busy
   crate::error::record_ignored(
       crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(instance)),
       "check and dispatch enqueued rerun prompt immediately if idle",
   );
   ```
3. Update the immediate Telegram confirmation message (lines 170–178):
   Change `"✅ Prompt dispatched to IDE!"` to `"✅ Prompt enqueued for execution (FIFO queue)!"`.
4. Update `image_note` rendering (lines 187–193):
   ```rust
   let image_note = enqueued
       .image_payload
       .as_deref()
       .map(str::trim)
       .filter(|payload| !payload.is_empty())
       .map(|_| "\n• <b>Image:</b> <i>original image attachment preserved and queued</i>")
       .unwrap_or("");
   ```
5. Update `reply` text formatting (lines 194–208):
   Update the card header to `🔁 <b>Prompt Re-Run Enqueued</b>`, displaying the enqueued queue ID `enqueued.id`, status `queued`, machine label, instance, conversation, and prompt preview.

---

### Step 3: Refactoring `src-tauri/src/modules/telegram_inbound/send_flow.rs`
In `dispatch_turn_rerun_execution(...)`:
1. Locate lines 1177–1186:
   ```rust
   let args = format!("{} --instance {} {}", full_cid, instance_id, injected_text);
   let injected = execute_prompt_injection(&args).await;

   // Trigger direct CDP submission for instance
   let _cdp_ok = crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
       instance_id,
       &injected_text,
       true,
   )
   .is_ok();
   ```
2. Replace this block entirely with `enqueue_prompt_for_instance`:
   ```rust
   let mut enqueued = match crate::modules::repo_db::enqueue_prompt_for_instance(
       instance_id,
       repo_path,
       &injected_text,
       Some(full_cid),
   ) {
       Ok(p) => p,
       Err(err) => {
           let action_label = if is_verify { "verification" } else { "turn rerun" };
           return (
               format!(
                   "⚠️ <b>Failed to enqueue {}:</b> <code>{}</code>",
                   action_label,
                   clean_for_telegram_html(&err, 200)
               ),
               None,
           );
       }
   };

   // Preserve or update image payload if media detected
   let (_, detected_images) =
       crate::modules::repo_db::extract_image_payload_or_path(&injected_text);
   if !detected_images.is_empty() {
       enqueued.image_payload = Some(detected_images.join(";"));
       // Justification: Persisting detected media paths to SQLite queue is best-effort cache sync
       crate::error::record_ignored(
           crate::modules::repo_db::save_or_requeue_prompt(&enqueued),
           "persist media paths for turn rerun prompt",
       );
   }

   // Wake background scheduler immediately
   crate::modules::scheduler::notify_prompt_queue();

   // Immediate dispatch check if the instance is currently idle
   // Justification: Idle dispatch check is best-effort; background scheduler picks up prompt if busy
   crate::error::record_ignored(
       crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(instance_id)),
       "check and dispatch enqueued turn rerun prompt immediately if idle",
   );
   ```
3. Update the response card formatting (lines 1205–1240):
   Change `{icon} <b>Prompt {action} Dispatched & Acknowledged</b>` to:
   `{icon} <b>Prompt {action} Enqueued in FIFO Queue</b>`.
   Add line: `• <b>Status:</b> <code>queued</code> (Queue ID: <code>{queue_id}</code>)\n`.

---

## 4. Verification Contracts & Parameter Mappings

### 4.1 Parameter Mapping Table
| Target Function | Inbound Parameter | Enqueue Mapping (`enqueue_prompt_for_instance`) |
|---|---|---|
| `prompt_history::rerun_prompt_by_id` | `prompt.instance_id` | `instance_id: &str` (normalized or `"default"`) |
| `prompt_history::rerun_prompt_by_id` | `prompt.repo_path` | `repo_path: &str` |
| `prompt_history::rerun_prompt_by_id` | `prompt.prompt_content` | `prompt_content: &str` |
| `prompt_history::rerun_prompt_by_id` | `rerun_conversation(&prompt)` | `conversation_id: Option<&str>` |
| `send_flow::dispatch_turn_rerun_execution` | `instance_id` | `instance_id: &str` |
| `send_flow::dispatch_turn_rerun_execution` | `repo_path` | `repo_path: &str` |
| `send_flow::dispatch_turn_rerun_execution` | `injected_text` | `prompt_content: &str` |
| `send_flow::dispatch_turn_rerun_execution` | `full_cid` | `conversation_id: Option<&str>` |

### 4.2 Error Propagation & Safety Guardrails
- If `enqueue_prompt_for_instance` returns `Err(err)`, the handler returns an HTML error notification to Telegram and does NOT trigger downstream side effects.
- Best-effort operations (`save_or_requeue_prompt`, `send_telegram_message`, `check_and_dispatch_enqueued_prompts`) MUST use `crate::error::record_ignored` with documented justification comments.

---

## 5. Quality Gate & Pre-Flight Checks

Before marking this subtask complete, Worker 02 must run:
1. `cd src-tauri && cargo fmt -- --check`
2. `cd src-tauri && cargo clippy --lib --bin agm`
3. Verify that zero compiler errors (`E0428` or type mismatches) occur.
4. Verify that no absolute filesystem paths exist in changed lines.

# Subtask 02 — Refactor Transcript, Inbound Telegram, and Lease Conditionals

> **Subtask Code:** `Task-02`  
> **Subtask Name:** `02-refactor-transcript-and-inbound`  
> **Status:** `PENDING`  
> **Assigned Worker:** `Worker 02`  
> **Owned Files (Exclusive):**  
> - `src-tauri/src/modules/repo_db/transcript.rs`  
> - `src-tauri/src/modules/telegram_inbound/send_flow.rs`  
> - `src-tauri/src/modules/workspace_lease_manager/acquire.rs`  
> **Read-Only Context Files:**  
> - `02-spec/21-app/boolean-and-complex-conditions-remediation/02-component-spec.md`  
> - `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`  
> - `src-tauri/src/modules/repo_db/types.rs`  
> - `src-tauri/src/modules/telegram_inbound/types.rs`  
> - `src-tauri/src/modules/workspace_lease_manager/types.rs`  
> **Verification Command:** `cargo test --lib modules::telegram_inbound` (Exit code: 0)  

---

## 1. Intent & Scope

Refactor boolean conditionals, multi-clause compound expressions, mixed polarity checks, and negative identifiers across backend Rust modules `transcript.rs`, `send_flow.rs`, and `acquire.rs`. Decompose complex conditions into affirmative predicate helper functions adhering strictly to `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/` (Principle 4: 2-operand limit, Principle 5: implicit evaluation, Principle 6: positive framing).

### In-Scope Deliverables:

1. **`src-tauri/src/modules/repo_db/transcript.rs`**:
   - Extract and unit-test 9 affirmative predicate helpers:
     - `is_subagent_prompt_text(txt: &str) -> bool`
     - `is_telemetry_event(s_type: &str) -> bool`
     - `is_tool_execution_step(s_type: &str, s_source: &str, val: &serde_json::Value) -> bool`
     - `is_terminal_status(status: &str) -> bool`
     - `is_planner_turn_completed(has_open_tools: bool, s_status: &str) -> bool`
     - `is_genuine_user_input(s_type: &str, s_source: &str) -> bool`
     - `is_system_prompt_message(s_source: &str, content: &str) -> bool`
     - `is_content_truncated(val: &serde_json::Value) -> bool`
     - `is_relevant_turn_candidate(trimmed: &str) -> bool`
   - Refactor `inspect_first_line_is_subagent` using `is_subagent_prompt_text`.
   - Refactor `inspect_conversation_transcript` across both forward and reverse scans using the extracted helpers, eliminating inline duplicated logic.
   - Refactor `extract_conversation_prompt_turns` to use `is_relevant_turn_candidate`, `is_genuine_user_input`, `is_system_prompt_message`, and `is_planner_turn_completed`.

2. **`src-tauri/src/modules/telegram_inbound/send_flow.rs`**:
   - Extract and unit-test 8 affirmative predicate helpers:
     - `is_valid_send_token(token: &str) -> bool` (replaces compound negative `picker_safe`)
     - `is_machine_entry_usable(label: &str) -> bool` (standardizes affirmative naming)
     - `is_known_ip(ip: &str) -> bool`
     - `is_matching_target_machine(machine_candidate: &str, target: &str) -> bool`
     - `is_duplicate_send_machine_tap(existing_machine: &str, candidate_alias: &str, stage: SendStage, elapsed_secs: u64) -> bool`
     - `is_matching_machine_alias(entry_alias: &str, input_alias: &str) -> bool`
     - `is_instance_mismatch(prompt_instance_id: &str, target_instance_id: &str) -> bool`
     - `is_likely_prompt_text(trimmed: &str) -> bool`
   - Refactor `format_instance_choice` to use `is_known_ip`.
   - Refactor `on_machine_picked` to use `is_duplicate_send_machine_tap` and `is_matching_machine_alias`.
   - Refactor `recent_prompts_for_destination` to use `is_instance_mismatch`.
   - Refactor `consume_send_prompt` to decompose prompt text checks into guard clauses and use `is_likely_prompt_text`.
   - Update existing unit tests (`picker_safety_rejects_separators`, `machine_entry_usable_guards_picker_payloads`) to verify new helpers.

3. **`src-tauri/src/modules/workspace_lease_manager/acquire.rs`**:
   - Extract affirmative predicate helper:
     - `is_lease_held_by_other_node(expires: i64, now: i64, owner: &str, node_id: &str) -> bool`
   - Refactor REST fallback lease collision check in `acquire_lease_with_details`.

---

## 2. Step-by-Step Execution Plan

### Step 1: Refactor `src-tauri/src/modules/repo_db/transcript.rs`

1. Define affirmative predicate helper functions near the top of `transcript.rs`:
   - `is_subagent_prompt_text(txt: &str) -> bool`
   - `is_telemetry_event(s_type: &str) -> bool`
   - `is_tool_execution_step(s_type: &str, s_source: &str, val: &serde_json::Value) -> bool`
   - `is_terminal_status(status: &str) -> bool`
   - `is_planner_turn_completed(has_open_tools: bool, s_status: &str) -> bool`
   - `is_genuine_user_input(s_type: &str, s_source: &str) -> bool`
   - `is_system_prompt_message(s_source: &str, content: &str) -> bool`
   - `is_content_truncated(val: &serde_json::Value) -> bool`
   - `is_relevant_turn_candidate(trimmed: &str) -> bool`

2. Refactor `inspect_first_line_is_subagent` (lines 47–54):
   - Replace 8-clause inline returns with `is_subagent_prompt_text(txt)`.

3. Refactor `inspect_conversation_transcript`:
   - Replace first reverse scan telemetry filter (lines 279–287) with `if is_telemetry_event(s_type) { continue; }`.
   - Replace tool execution step check (lines 291–296) with `if is_tool_execution_step(s_type, s_source, &val) { continue; }`.
   - Replace terminal planner completion check (lines 306–320) with `if is_planner_turn_completed(has_tool_calls, s_status) { is_completed_or_waiting = true; break; }`.
   - Replace terminal status check (lines 324–327) with `if is_terminal_status(s_status) { is_completed_or_waiting = true; break; }`.
   - Replace user input check (lines 358–361) with `let is_user_input = is_genuine_user_input(s_type, s_source);`.
   - Replace system message check (lines 367–374) with `let is_system_msg = is_system_prompt_message(s_source, c);`.
   - Replace truncated fields traversal (lines 376–380) with `if is_content_truncated(&val) && transcript_full_path.exists()`.
   - Replace subagent text check (line 402) with `if is_subagent_prompt_text(c_trim) { is_subagent = true; }`.
   - Replace response truncated fields traversal (lines 424–428) with `if is_content_truncated(&val) && transcript_full_path.exists()`.
   - Replace tool output check (lines 450–455) with `if is_tool_execution_step(s_type, s_source, &val)`.
   - Replace deep reverse scan telemetry filter (lines 476–484) with `if is_telemetry_event(s_type) { continue; }`.
   - Replace deep reverse scan tool step check (lines 488–495) with `if is_tool_execution_step(s_type, s_source, &val) { continue; }`.
   - Replace deep reverse scan terminal status check (lines 551–554) with `let is_explicit_done = is_terminal_status(s_status);`.
   - Replace fallback user input check (lines 658–661) with `let is_user = is_genuine_user_input(s_type, s_source);`.

4. Refactor `extract_conversation_prompt_turns`:
   - Replace line probe filter (lines 763–773) with `if !is_relevant_turn_candidate(trimmed) { continue; }`.
   - Replace user input check (lines 788–791) with `let is_user_input = is_genuine_user_input(s_type, s_source);`.
   - Replace system message check (lines 797–804) with `let is_internal = is_system_prompt_message(s_source, c);`.
   - Replace planner done check (lines 879–882) with `let is_done = is_planner_turn_completed(has_open_tools, s_status);`.

### Step 2: Refactor `src-tauri/src/modules/telegram_inbound/send_flow.rs`

1. Define affirmative predicate helper functions in `send_flow.rs`:
   - `is_valid_send_token(token: &str) -> bool`:
     - Replaces `picker_safe(token: &str) -> bool`.
     - Returns `false` on empty; decomposes separator checks into affirmative boolean `has_separator`.
   - `is_machine_entry_usable(label: &str) -> bool`:
     - Replaces `machine_entry_usable`.
     - Checks `is_valid_send_token(label)` and `callback_fits(&format!("send:m:{label}"))`.
   - `is_known_ip(ip: &str) -> bool`:
     - Returns `false` if trimmed string is empty or matches `"unknown ip"`.
   - `is_matching_target_machine(machine_candidate: &str, target: &str) -> bool`:
     - Compares candidates with case-insensitive equality or both-current resolution.
   - `is_duplicate_send_machine_tap(existing_machine: &str, candidate_alias: &str, stage: SendStage, elapsed_secs: u64) -> bool`:
     - Encapsulates 3-second debounce window check.
   - `is_matching_machine_alias(entry_alias: &str, input_alias: &str) -> bool`:
     - Encapsulates alias equality and "local" / "localhost" resolution.
   - `is_instance_mismatch(prompt_instance_id: &str, target_instance_id: &str) -> bool`:
     - Replaces negative triple-check in recent prompts filter.
   - `is_likely_prompt_text(trimmed: &str) -> bool`:
     - Evaluates word count and command action prefixes cleanly.

2. Update call sites across `send_flow.rs`:
   - Update `machine_entries`: replace `machine_entry_usable` calls with `is_machine_entry_usable`.
   - Update `format_instance_choice`: replace `!ip.is_empty() && ip != "Unknown IP"` with `is_known_ip(ip)`.
   - Update `on_machine_picked`:
     - Replace lines 552–558 with `is_duplicate_send_machine_tap(&existing.machine, alias, existing.stage, elapsed)`.
     - Replace lines 569–573 with `is_matching_machine_alias(e, alias)`.
   - Update `recent_prompts_for_destination`:
     - Replace lines 727–729 with `if is_instance_mismatch(&p.instance_id, instance) { return false; }`.
   - Update `consume_send_prompt`:
     - Replace line 1313 with discrete early guard on `/` prefix, followed by `has_text || has_media` evaluation.
     - Replace lines 1354–1358 with `let is_likely_prompt = is_likely_prompt_text(trimmed);`.

3. Update unit tests in `send_flow.rs`:
   - Update `picker_safety_rejects_separators` to test `is_valid_send_token`.
   - Update `machine_entry_usable_guards_picker_payloads` to test `is_machine_entry_usable`.

### Step 3: Refactor `src-tauri/src/modules/workspace_lease_manager/acquire.rs`

1. Define affirmative predicate helper function in `acquire.rs`:
   ```rust
   pub(crate) fn is_lease_held_by_other_node(
       expires: i64,
       now: i64,
       owner: &str,
       node_id: &str,
   ) -> bool {
       let is_unexpired = expires > now;
       if !is_unexpired {
           return false;
       }
       let is_same_node = owner.trim().eq_ignore_ascii_case(node_id.trim());
       !is_same_node
   }
   ```

2. Update REST fallback collision guard in `acquire_lease_with_details` (lines 171–173):
   - Replace `if current_expires > now && !current_owner.trim().eq_ignore_ascii_case(node_id.trim())` with:
     ```rust
     if is_lease_held_by_other_node(current_expires, now, current_owner, &node_id) {
     ```

### Step 4: Verification & Quality Gate

1. Verify Rust formatting:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. Run comprehensive Clippy gate:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. Run telegram inbound test suite:
   ```bash
   cargo test --lib modules::telegram_inbound
   ```
4. Run repo db test suite:
   ```bash
   cargo test --lib modules::repo_db
   ```
5. Confirm exit code 0 for all commands.

---

## 3. Negative Boundaries & Invariants

- **TOTAL BAN ON GIT COMMANDS:** Subagents and workers NEVER run `git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`, etc.
- **Strict Disjoint Ownership:** Only edit `src-tauri/src/modules/repo_db/transcript.rs`, `src-tauri/src/modules/telegram_inbound/send_flow.rs`, and `src-tauri/src/modules/workspace_lease_manager/acquire.rs`. Never touch frontend files or other backend modules.
- **Strict Relative Git Paths:** Only repository-relative paths in code, comments, plans, and commit messages. Never write absolute paths or `file:///` URIs.
- **Preserve System Invariants:**
  - Do not alter the 10-minute thinking window or the 25-line deep transcript reverse scan depth.
  - Do not alter the 3-second debounce window in Telegram inbound machine picking.
  - Do not alter the 60-byte callback payload budget in Telegram pickers.
  - Do not alter the lease TTL fallback rules (90s floor, 1800s default).

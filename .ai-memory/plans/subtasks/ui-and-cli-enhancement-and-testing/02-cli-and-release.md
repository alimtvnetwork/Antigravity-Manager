# Subtask 02: CLI Command Dispatch, Sequence Resolution, CDP Fallback & Minor Release Bump

- **Subtask ID**: `Task-02`
- **Parent Task**: `ui-and-cli-enhancement-and-testing`
- **Spec Reference**: `02-spec/21-app/ui-and-cli-enhancement-and-testing/02-cli-and-copy-spec.md`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: `READY_FOR_EXECUTION`

---

## 1. Subtask Objective & Scope

Task-02 owns the CLI enhancements, AGM sequence resolution, direct CDP prompt dispatch fallback, dual MIME rich clipboard utility, and the official minor release bump ceremony (`npm run bump minor`) for version `v4.204.0`.

### Owned Files
- `src-tauri/src/bin/agm/main.rs` (subcommand matcher and routing)
- `src-tauri/src/bin/agm/prompt_cmds.rs` (prompt domain dispatch, sequence resolution in send/queue)
- `src-tauri/src/bin/agm/prompt_dispatch.rs` (`--queue` flag handling and sequence resolution)
- `src-tauri/src/bin/agm/common.rs` (`is_prompt_subcommand` alias normalization)
- `src-tauri/src/bin/agm/help_text.rs` (CLI command tables and JSON help descriptors)
- `src-tauri/src/modules/repo_db/agy.rs` (direct CDP fallback in `send_prompt_now_for_instance`)
- `src/components/instances/PromptTreeView/clipboard.ts` (dual MIME `ClipboardItem` utility)
- `package.json`, `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md` (release bump artifacts)

---

## 2. Step-by-Step Implementation Tasks

### Task 2.1: CLI Subcommand Routing & Alias Normalization
- **File**: `src-tauri/src/bin/agm/common.rs`
  - In `is_prompt_subcommand`, add `"enqueue"` to the recognized prompt domain subcommands:
    ```rust
    pub(crate) fn is_prompt_subcommand(sub: &str) -> bool {
        let s = sub.trim_start_matches('/').trim_start_matches('-').to_lowercase();
        matches!(
            s.as_str(),
            "list" | "ls" | "tree" | "inspect" | "observe" | "send" | "dispatch"
                | "running" | "wpr" | "queue" | "enqueue" | "qe" | "history"
                | "export" | "backup" | "brp" | "restore" | "rrp" | "purge" | "clean"
        )
    }
    ```
- **File**: `src-tauri/src/bin/agm/main.rs`
  - In `main()`, ensure `"queues" | "queue" | "enqueue" | "qe"` match arms route to `crate::prompt_cmds::dispatch_prompt_domain("queue", &cmd_args);`.
  - In `prompt` domain routing, ensure arguments starting with `send`, `queue`, or `enqueue` route directly to `prompt_cmds::dispatch_prompt_domain_from_args`.
- **File**: `src-tauri/src/bin/agm/prompt_cmds.rs`
  - Update `dispatch_prompt_domain` to map `"queue" | "enqueue" | "qe"` to `cmd_prompts_queue`.

### Task 2.2: AGM Sequence Resolution for `cmd_prompts_send` & `cmd_prompts_queue`
- **File**: `src-tauri/src/bin/agm/prompt_cmds.rs`
  - In `cmd_prompts_send`:
    - Check if the first positional argument is an AGM sequence identifier (`C001`, `P001`, `AGM:C001`, `GM:#1`, etc.) via `repo_db::resolve_agm_sequence_target`.
    - If resolved:
      - Assign `resolved_inst = seq.instance_id`.
      - Assign `repo_path = seq.repo_path`.
      - Assign `session_id = seq.conversation_id.or(session_id)`.
      - Strip the sequence token from positional arguments so it is not duplicated in prompt content.
  - In `cmd_prompts_queue`:
    - Apply identical sequence target resolution via `repo_db::resolve_agm_sequence_target`.
    - Extract target instance, repo path, and conversation ID before enqueuing to SQLite.

### Task 2.3: Support `--queue` Flag in `cmd_prompt_dispatch`
- **File**: `src-tauri/src/bin/agm/prompt_dispatch.rs`
  - In argument loop of `cmd_prompt_dispatch`, parse `--queue` / `-q` flag:
    ```rust
    let mut is_queue_mode = false;
    // ...
    } else if arg == "--queue" || arg == "-q" {
        is_queue_mode = true;
        i += 1;
        continue;
    }
    ```
  - When `is_queue_mode` is `true`:
    - Route prompt through `repo_db::enqueue_prompt_for_instance(&inst_id, &cwd_str, &final_prompt, Some(&session_id))` instead of invoking `spawn_or_replace_prompt_via_agy`.
    - Output `[SUCCESS] Prompt queued for instance '{}' (conv: {}).` or structured `CliEnvelope::ok`.

### Task 2.4: Direct DevTools CDP Insertion Fallback in `repo_db/agy.rs`
- **File**: `src-tauri/src/modules/repo_db/agy.rs`
  - In `send_prompt_now_for_instance`:
    - When `spawn_or_replace_prompt_via_agy(&active_prompt, true)` returns an `Err(spawn_err)` (such as when `agy` binary is not found on `PATH` or process spawning fails):
      - Extract clean user prompt: `let clean_prompt = extract_clean_user_prompt(prompt_content);`.
      - Invoke `crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(&canonical_inst, &clean_prompt)`.
      - If CDP submission returns `true`:
        - Log fallback success to logger: `Direct CDP fallback succeeded for prompt '{}' on instance '{}'`.
        - Record audit event via `audit::record_if_active("backend", "repo_db", "cdp_dispatch_fallback", ...)`.
        - Return `Ok(active_prompt)` without propagating an error.
      - If CDP submission returns `false`:
        - Focus IDE window using `ensure_instance_running_for_dispatch`.
        - Return descriptive error: `Err(format!("{spawn_err}; direct CDP fallback failed; {focus_note}"))`.

### Task 2.5: Dual MIME Clipboard Copy Verification & Hardening
- **File**: `src/components/instances/PromptTreeView/clipboard.ts`
  - Verify `copyPromptWithRichImages` transforms:
    - `\n\n` to `</p><p>`
    - `\n` to `<br/>`
    - `![(.*?)\]\((.*?)\)` to `<img src="$2" alt="$1" style="max-width:100%; border-radius:8px; margin:8px 0;" />`
    - Full HTML document wrapping.
  - Verify `ClipboardItem` constructs with both `'text/html'` and `'text/plain'` Blobs.
  - Verify error handling routes through `useErrorStore.getState().trackWarning` before plaintext fallback.

### Task 2.6: CLI Help Text & Documentation Update
- **File**: `src-tauri/src/bin/agm/help_text.rs`
  - Update `print_commands_table`:
    - Add `prompt send [target] "<text>"` documentation.
    - Add `prompt queue [target] "<text>"` and `queue [target] "<text>"` documentation.
    - Add `--queue` flag documentation under `prompt`.
  - Update `print_help_json`:
    - Add `prompt send` and `prompt queue` command entries with their respective aliases and flags.

### Task 2.7: Pre-flight Quality Gates & Minor Version Release Ceremony
- **Pre-flight Quality Checks**:
  1. `cd src-tauri && cargo fmt -- --check`
  2. `cd src-tauri && cargo clippy --all-targets --all-features`
  3. `npm run build`
- **Minor Bump Execution**:
  - Run `npm run bump minor` (advances version from `4.203.0` to `4.204.0`).
- **Release Documentation Synchronization**:
  - `CHANGELOG.md`: Fill release skeleton for `v4.204.0` with strict attribution `(Thanks to @aukgit)`.
  - `CHANGELOG_EN.md`: Fill English release skeleton for `v4.204.0` with strict attribution `(Thanks to @aukgit)`.
  - `README.md`: Update `## 📝 更新日志` with stable release summary for `v4.204.0`.
  - `README_EN.md`: Update `## 📝 Changelog` with stable English release summary for `v4.204.0`.

---

## 3. Detailed Verification Procedures

### Test 1: CLI Prompt Dispatch & Queue Verification
```bash
# 1. Test direct dispatch syntax
agm prompt send C001 "Test immediate prompt dispatch"

# 2. Test queue subcommand syntax
agm prompt queue P001 "Test FIFO prompt queueing"

# 3. Test top-level queue shortcuts
agm queue "Test top level queue shortcut"
agm enqueue "Test top level enqueue alias"

# 4. Test prompt dispatch with --queue flag
agm prompt C001 "Test dispatch with queue flag" --queue

# 5. Verify JSON output envelope
agm prompt send C001 "Test JSON output" --json
agm prompt queue P001 "Test JSON queue" --json
```

### Test 2: CDP Direct Fallback Verification
1. Temporarily rename or mock `agy` binary to simulate headless/missing environment.
2. Execute `agm prompt send "Verify direct CDP fallback"`.
3. Inspect Antigravity IDE:
   - Message composer receives prompt text via `Input.insertText`.
   - Send button is clicked automatically.
   - Command returns success and logs `[RepoDB] Direct CDP fallback succeeded`.

### Test 3: Dual MIME Clipboard Copy Verification
1. Open Prompt Tree View in GUI.
2. Inspect a prompt containing markdown image syntax (`![Architecture](file:///path/to/img.png)`).
3. Click "Copy All (Content + Images)".
4. Paste into rich text target (e.g., Slack / Word / Google Docs):
   - Images render as inline HTML elements.
5. Paste into plain text target (e.g., Notepad / VS Code):
   - Content renders as clean markdown text.

### Test 4: Pre-flight & Release Bump Verification
1. Confirm zero warnings from `cargo clippy --all-targets --all-features`.
2. Confirm zero formatting drift from `cargo fmt -- --check`.
3. Confirm clean frontend build from `npm run build`.
4. Run `npm run bump minor` and verify all 15 files are synchronized to `v4.204.0`.
5. Verify attribution strictly credits `@aukgit` and no other user handles appear.

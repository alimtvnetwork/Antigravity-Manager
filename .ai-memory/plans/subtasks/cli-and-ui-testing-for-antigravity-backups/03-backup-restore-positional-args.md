# Subtask 03: Positional Instance Argument Resolution & Real-Time Multimodal Step Tracing

## 1. Overview & Objectives

Upgrade `src-tauri/src/bin/agm/running_backup_cmds.rs` to support flexible, human-friendly command invocations:
- `agm backup running [instance]`
- `agm restore running [instance]`
- `agm backup #1`, `agm backup #2`, `agm backup worker-1`, `agm backup a-6650`
- `agm restore #1`, `agm restore default`, `agm restore running #2`

Key Objectives:
1. Implement `parse_instance_and_noise_args(args: &[String], is_restore: bool)` to strip conversational noise tokens (`running`, `running-prompts`, `rp`, `prompts`) and extract positional instance identifiers.
2. Route instance specifiers through `antigravity_tools_lib::modules::instance::resolve_instance_id` to resolve sequence numbers (`#1`, `1`), IDs (`inst-6650`), names, and suffix matches (`a-6650`).
3. Add real-time, multi-step CLI tracing (`[Step 1/3]`, `[Step 2/3]`, `[Step 3/3]`) detailing each prompt turn, Base64 image payload, and referenced screenshot file during both backup and restoration.
4. Enforce strict FIFO restoration ordering (`ORDER BY created_at ASC, id ASC`) and caller session protection (`guard_caller_instance`).

---

## 2. Files Involved

- `src-tauri/src/bin/agm/running_backup_cmds.rs`: Primary command logic, argument normalization, and progress logging.
- `src-tauri/src/bin/agm/prompt_cmds.rs`: Forwarding entry points `cmd_prompts_backup` and `cmd_prompts_restore`.
- `src-tauri/src/bin/agm/main.rs`: Top-level CLI routing aliases for `backup` and `restore`.
- `src-tauri/src/modules/backup_prompts_db.rs`: Split SQLite snapshot queries and batch restoration.
- `src-tauri/src/modules/repo_db/dispatch.rs`: FIFO re-injection into active prompts.

---

## 3. Detailed Implementation Plan

### Step 1: Implement `parse_instance_and_noise_args`
In `src-tauri/src/bin/agm/running_backup_cmds.rs`, define the argument parsing helper:

```rust
pub(crate) struct NormalizedBackupArgs {
    pub target_instance: Option<String>,
    pub custom_file: Option<String>,
    pub is_json: bool,
    pub is_keep: bool,
    pub is_force: bool,
    pub is_help: bool,
    pub action: Option<String>, // e.g., "ls", "clean"
}

pub(crate) fn parse_instance_and_noise_args(
    args: &[String],
    is_restore: bool,
) -> NormalizedBackupArgs
```

**Parsing Logic:**
1. Check for help flags (`help`, `--help`, `-h`).
2. Recognize subcommands (`ls`, `list`, `clean`).
3. Parse standard flags:
   - `-f`, `--file`, `-file` -> `custom_file`
   - `-i`, `--instance`, `-instance`, `--instance=...`, `-i=...` -> explicit `target_instance`
   - `--json`, `-j` -> `is_json`
   - `--keep`, `-k` -> `is_keep`
   - `--force` -> `is_force`
4. For remaining tokens:
   - Filter out noise tokens: `running`, `running-prompts`, `rp`, `prompts`, `active`.
   - Take the first remaining positional token as a potential instance specifier.
   - Resolve through `instance::resolve_instance_id(&token)`.
   - On backup: If no instance is specified, fall back to `instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())`.
   - On restore: If no instance is specified, remain `None` (representing an all-instances restore scope).

### Step 2: Integrate Real-Time Multimodal Step Tracing in `cmd_backup_running_prompts`
Replace the monolithic backup output with phased step logging:
- **Phase 1 (`[Step 1/3]`)**: Log target instance, project directories, and total detected active prompts.
- **Phase 2 (`[Step 2/3]`)**: Iterate through prompts being backed up. For each prompt:
  - Print sequence `#`, short ID, project name.
  - Inspect `image_payload`: check for embedded Base64 data and referenced files.
  - Trace image details: byte size, MIME type, sub-image counts.
- **Phase 3 (`[Step 3/3]`)**: Commit batch to split SQLite (`backup_prompts.db`). Print batch ID and confirmation summary.

### Step 3: Integrate Real-Time Multimodal Step Tracing in `cmd_restore_running_prompts`
Refactor restore execution:
- **Phase 1 (`[Step 1/3]`)**: Query unrestored batches for target instance scope (or all). Print batch IDs found.
- **Phase 2 (`[Step 2/3]`)**: For each restored prompt record:
  - Print FIFO sequence number (`Turn X of Y`).
  - Trace reconstituted image payloads (Base64 payload restoration and workspace file reference re-anchoring).
  - Seed workspace task resumption file (`.antigravity_resume_task.<inst>.json`).
- **Phase 3 (`[Step 3/3]`)**: Re-enqueue into `active_prompts` in strict FIFO order (`created_at ASC, id ASC`). Mark batch as restored (initiating 1-day retention timer).

### Step 4: Ensure Caller IDE Protection
Before executing any destructive or switching operations triggered in combination with backup/restore:
- Verify instance target via `crate::instance_cmds::guard_caller_instance(args, &target_instance)`.
- Disallow killing or terminating caller session (`default-copy-5597`, PID 11588).

---

## 4. Verification & Testing

1. **Unit Test for Parser**:
   - `parse_instance_and_noise_args(&["running", "#1"], false)` -> resolves instance `#1`.
   - `parse_instance_and_noise_args(&["running", "worker-1"], false)` -> resolves `worker-1`.
   - `parse_instance_and_noise_args(&["rp", "a-6650"], true)` -> resolves `a-6650`.
   - `parse_instance_and_noise_args(&["running"], true)` -> returns `None` (all instances).
2. **CLI Invocation Tests**:
   - Execute `cargo test -p antigravity-tools-lib` or targeted CLI checks.
   - Run `agm backup running #1` against test sandboxes.
   - Run `agm restore running #1` against test sandboxes.
3. **Trace Log Verification**:
   - Verify presence of `[Step 1/3]`, `[Step 2/3]`, and `[Step 3/3]` in terminal stdout.
   - Verify image count and sub-image details are printed correctly.

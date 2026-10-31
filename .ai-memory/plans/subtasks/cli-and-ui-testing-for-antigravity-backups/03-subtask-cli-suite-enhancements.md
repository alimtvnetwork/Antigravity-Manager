# Subtask 03: CLI Suite Enhancements (`agm running`, `agm backup/restore running`)

> **Subtask File:** `.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/03-subtask-cli-suite-enhancements.md`  
> **Parent Plan:** `.ai-memory/plans/pending/cli-and-ui-testing-for-antigravity-backups.md`  
> **Spec Reference:** [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-component-spec.md](../../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-component-spec.md)  
> **State:** `[QUEUED]`  
> **Assignee:** Backend CLI Engineer  

---

## 1. Objective & Scope

Implement CLI command routing and execution enhancements for running prompt and instance inspection in the native `agm` terminal binary:
1. Route `agm running [--json]` to an integrated instance and running prompt inspection engine.
2. Support `agm backup running [instance]` and `agm restore running [instance]` with explicit 4-step progress logging.
3. Protect caller developer sessions (`default-copy-5597`) during backup and restoration.
4. Guarantee strict First-In-First-Out (`FIFO`) ordering during restore queries.

---

## 2. Target Files & Symbols

| File | Type of Change | Key Symbols / Locations |
| :--- | :--- | :--- |
| `src-tauri/src/bin/agm/main.rs` | Router Modification | `match subcommand.as_str()` branches `"active" \| "running"`, `"backup"`, `"restore"` |
| `src-tauri/src/bin/agm/running_prompts_cmds.rs` | Function Addition / Enhancement | `pub(crate) fn cmd_running_inspection(args: &[String])` |
| `src-tauri/src/bin/agm/running_backup_cmds.rs` | Function Enhancement | `pub(crate) fn cmd_backup_running_prompts`, `pub(crate) fn cmd_restore_running_prompts` |

---

## 3. Implementation Steps

### Step 3.1: Router Updates in `src-tauri/src/bin/agm/main.rs`
1. Locate the `"active" | "running"` match branch in `main.rs`.
2. Replace the previous tree/agy delegation with a direct call to `cmd_running_inspection(&cmd_args)`:
   ```rust
   "active" | "running" => {
       crate::running_prompts_cmds::cmd_running_inspection(&cmd_args);
   }
   ```
3. Update `"backup"` / `"backup-running-prompts"` to intercept the `"running"` or `"rp"` subcommand:
   ```rust
   "backup-running-prompts" | "brp" | "backup-prompts" | "backup" | "backpack" => {
       if cmd_args.first().map(|s| s.eq_ignore_ascii_case("running") || s.eq_ignore_ascii_case("rp")).unwrap_or(false) {
           crate::running_backup_cmds::cmd_backup_running_prompts(&cmd_args[1..]);
       } else {
           crate::prompt_cmds::dispatch_prompt_domain("backup", &cmd_args);
       }
   }
   ```
4. Update `"restore"` / `"restore-running-prompts"` similarly:
   ```rust
   "restore-running-prompts" | "rrp" | "restore-prompts" | "restore" => {
       if cmd_args.first().map(|s| s.eq_ignore_ascii_case("running") || s.eq_ignore_ascii_case("rp")).unwrap_or(false) {
           crate::running_backup_cmds::cmd_restore_running_prompts(&cmd_args[1..]);
       } else {
           crate::prompt_cmds::dispatch_prompt_domain("restore", &cmd_args);
       }
   }
   ```

### Step 3.2: Implement `cmd_running_inspection` in `running_prompts_cmds.rs`
1. Parse flags: `--json`, `-j`, `--words <N>`, `--limit <Y>`, and positional `[instance]`.
2. Retrieve instances list via `instance::list_instances()`. For each instance, check if running (`instance::is_instance_running(id)`), resolve PID, and match active prompts.
3. Query `repo_db::list_all_prompts()` filtering for `running`, `queued`, and `dispatched` statuses.
4. Truncate prompt preview text using `crate::common::truncate_words(&p.prompt_content, max_words)`.
5. If `--json` is supplied, serialize instances and prompts matching the schema in `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-component-spec.md`.
6. Otherwise, print the ASCII summary table with aligned columns: `SEQ`, `ID`, `PROJECT`, `STATUS`, `IMAGES`, `WORDS`, `PREVIEW`.

### Step 3.3: 4-Step Progress Logging in `running_backup_cmds.rs`
1. In `cmd_backup_running_prompts`, check if the first argument is `"running"` or `"rp"`. If so, strip it.
2. If non-flag tokens exist, set `target_instance` to the first non-flag token.
3. If not in JSON mode, emit structured step logs:
   - `[Step 1/4] Resolving target instance and validating caller IDE isolation...`
     - Detect caller session using `instance::detect_caller_instance_id()`.
   - `[Step 2/4] Scanning in-flight running prompts and multimodal image payloads...`
     - Scan live prompts and count inline Base64 payloads and image file references.
   - `[Step 3/4] Persisting atomic snapshot records to split SQLite database...`
     - Invoke `backup_prompts_db::backup_active_running_prompts(Some(&target_instance), custom_file)`.
   - `[Step 4/4] Verifying snapshot integrity and recording batch metadata...`
     - Output confirmation and batch ID.

### Step 3.4: 4-Step Progress Logging in `cmd_restore_running_prompts`
1. In `cmd_restore_running_prompts`, strip `"running"` or `"rp"` if passed as the first token.
2. If not in JSON mode, emit structured step logs:
   - `[Step 1/4] Querying split SQLite database for unrestored backup records (FIFO ordering)...`
   - `[Step 2/4] Restoring prompt records into local SQLite execution queues (repo_db)...`
     - Enforce `ORDER BY created_at ASC, sequence_id ASC`.
   - `[Step 3/4] Seeding atomic workspace resumption task file (.antigravity_resume_task.<inst>.json)...`
     - Invoke `instance::seed_resume_task_file(&inst_id, &prompts)`.
   - `[Step 4/4] Activating 24-hour retention timer and broadcasting execution notifications...`

---

## 4. Verification & Quality Gates

Run the following commands to verify implementation:

1. **Compilation & Linter:**
   ```bash
   cd src-tauri && cargo clippy --bin agm
   cd src-tauri && cargo fmt -- --check
   ```

2. **Running Inspection CLI:**
   ```bash
   agm running
   agm running --json
   agm running default-copy-5597 --words 150
   ```
   *Expected:* Clean ASCII table or JSON output without delegating to `gitmap` or printing help banners.

3. **4-Step Backup Progress:**
   ```bash
   agm backup running default-copy-5597
   ```
   *Expected:* Output contains `[Step 1/4]`, `[Step 2/4]`, `[Step 3/4]`, `[Step 4/4]`.

4. **4-Step Restore Progress:**
   ```bash
   agm restore running default-copy-5597
   ```
   *Expected:* Output contains `[Step 1/4]`, `[Step 2/4]`, `[Step 3/4]`, `[Step 4/4]`.

---
Maintained by @aukgit

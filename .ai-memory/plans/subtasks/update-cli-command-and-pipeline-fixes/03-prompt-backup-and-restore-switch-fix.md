# Subtask Implementation Plan: Prompt Backup & Restore Fixes Across Switching and Rotation

> **Subtask ID:** `03-prompt-backup-and-restore-switch-fix`  
> **Parent Slug:** `update-cli-command-and-pipeline-fixes`  
> **Spec Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md)  
> **AI Instructions Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md)  
> **Target Files:**
> - `src-tauri/src/modules/auto_switcher/rotation.rs`
> - `src-tauri/src/modules/repo_db/backup.rs`
> - `src-tauri/src/modules/repo_db/auto_resume.rs`
> - `src-tauri/src/bin/agm/prompt_cmds.rs`
> - `src-tauri/src/bin/agm/instance_cmds.rs`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

During account rotation and profile switching, prompts running or in-flight on the depleted instance must be preserved and cleanly restored upon target instance relaunch. 

Three defects currently cause prompt loss and execution failure:
1. **Cross-Instance Prompt Amnesia**: When rotating across different instances (`target.instance_id != current_instance_id`), backed-up prompts in `active_prompts` retain `instance_id = current_instance_id`. Target instance launch queries for prompts attributed to `target.instance_id`, resulting in 0 matches and abandoned tasks.
2. **Restricted Backup & Auto-Resume Status Filters**: `backup_running_prompts` transitions only `status = 'running'`, leaving `'in_flight'` and `'dispatched'` prompts unbacked-up. Conversely, `auto_resume_recent_prompts` queries only `status IN ('queued', 'pending')`, skipping prompts in `'backed_up'`.
3. **CLI Dispatch Limitations**: `agm prompt send` and `agm prompt queue` lack options for targeting existing vs new conversations, default repository paths to `"scratch/test-repo"`, lack the `qe` alias, and `agm instance` lacks the `prompt` domain routing.

---

## 2. Concrete Implementation Steps

### Step 1: Expand Status Transitions in `src-tauri/src/modules/repo_db/backup.rs`
- In `backup_running_prompts`:
  - Update SQL transition queries to change `status` from `('running', 'in_flight', 'dispatched')` to `'backed_up'`.
  - Update `get_memory_prompts_map()` in-memory update loop to check `p.status == "running" || p.status == "in_flight" || p.status == "dispatched"`.

### Step 2: Include `'backed_up'` Status in `src-tauri/src/modules/repo_db/auto_resume.rs`
- In `auto_resume_recent_prompts`:
  - Update prompt query to `WHERE status IN ('queued', 'pending', 'backed_up')`.
  - Maintain strict FIFO ordering: `ORDER BY created_at ASC, id ASC LIMIT 1`.
  - Update task file verification: `task_status == "queued" || task_status == "pending" || task_status == "backed_up"`.

### Step 3: Implement Cross-Instance Prompt Re-Attribution in `src-tauri/src/modules/auto_switcher/rotation.rs`
- In `execute_profile_rotation_with_context`:
  - Inside the cross-instance branch `if target.instance_id != current_instance_id`:
  - Execute SQL query updating `active_prompts`:
    ```sql
    UPDATE active_prompts 
    SET instance_id = ?1, updated_at = ?2 
    WHERE instance_id = ?3 AND status IN ('backed_up', 'queued', 'pending')
    ```
  - Log successful re-attribution count via `logger::log_info`.

### Step 4: Add Flags and Defaults in `src-tauri/src/bin/agm/prompt_cmds.rs`
- Add `"qe"` alias for `"queue"` in `dispatch_prompt_domain`:
  `"queue" | "qe" => cmd_prompts_queue(args)`.
- In `cmd_prompts_send` and `cmd_prompts_queue`:
  - Replace `"scratch/test-repo"` fallback with `std::env::current_dir()`.
  - Parse `--new`, `--existing`, `--conversation <id>` flags and resolve `session_id`.

### Step 5: Route Prompt Domain in `src-tauri/src/bin/agm/instance_cmds.rs`
- In `dispatch_instance_domain`:
  - Add `"prompt" | "prompts" => crate::prompt_cmds::dispatch_prompt_domain_from_args(args)`.

---

## 3. Invariants & Guardrails

- **Relative Paths Only**: Zero absolute paths.
- **Positive Booleans**: `is_active`, `is_new_session`, `is_existing_session`.
- **FIFO Sorting**: `ORDER BY created_at ASC, id ASC`.
- **Zero Silent Errors**: Propagate errors or use `crate::error::record_ignored(..., "name")` with `// Justification:`.
- **Attribution**: Exclusively `@aukgit`.

---

## 4. Verification Checkpoints

1. Run `cargo check --bin agm` to verify CLI command compilation.
2. Run targeted test: `cargo test --lib modules::repo_db`.
3. Verify `agm prompt qe --help` or dispatch routes correctly.
4. Verify cross-instance prompt re-attribution SQL compiles without warnings.

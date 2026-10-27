# RCA 21: Switch restore gate parameter mismatch & 'in_progress' status omission

Read this before modifying account switching, prompt backup restore gates, or prompt queue dispatching.

## Incident
Across dozens of iterations, users observed that after a manual account switch or background auto-switch, running prompts that had been active before the switch were never restored into the newly launched IDE instance. SQLite backup snapshots contained the prompt, but the switch lifecycle reported:
`"No backed-up prompt for instance 'default'; skipping prompt restoration"`.

## Symptom
1. An IDE instance has a running or queued prompt.
2. Account switch is triggered (manual or auto-switcher).
3. The instance terminates, credentials rotate, and the new instance starts.
4. The prompt is never re-injected or dispatched to the IDE. The IDE opens with no active prompt.

## Grounded 4-Part Root Cause Analysis

### 1. Proximate Cause
In `src-tauri/src/modules/instance/switch.rs:676`, the post-switch restore gate checked:
```rust
let has_unrestored_backups = crate::modules::backup_prompts_db::list_prompt_backups(Some(&target_inst_id), None)
    .map(|records| !records.is_empty())
    .unwrap_or(false);
```
However, `list_prompt_backups` takes `batch_id: Option<&str>` as its first parameter and queries:
```sql
SELECT ... FROM prompt_backups WHERE backup_batch_id = ?1
```
Because batch IDs are formatted as `batch_<uuid>`, passing `&target_inst_id` (e.g. `"default"` or `"default-copy-5597"`) **always returned 0 rows**. `has_unrestored_backups` evaluated to `false`, and the entire prompt restore stage was skipped.

### 2. Root Cause Mechanism (The 5 Secondary Disconnects)
1. **Manual Switch Count Discard in `system_impl.rs`**:
   `DesktopIntegration::on_account_switch` called `backup_active_running_prompts_for_instance` which returned `(backed_up_count, backup_file)`. The return count was discarded with `let _ = ...`, and `needs_reinject` evaluated only `repo_db::count_backed_up_prompts("default")`.
2. **Missing Empty/NULL Scope in Default Instance Query**:
   `count_backed_up_prompts("default")` queried `instance_id = 'default' OR instance_id = '__default__'`. Unattributed default prompts with `instance_id = ''` or `NULL` returned 0.
3. **Restored Status Disconnect (`'in_progress'` vs `'queued'`/`'pending'`)**:
   `prompt_status_after_restore` mapped restored running prompts to `'in_progress'`. However:
   - `resend_single_pass` queried `status IN ('backed_up', 'queued', 'pending', 'running')` (missing `'in_progress'`).
   - `check_and_dispatch_enqueued_prompts` queried `status IN ('queued', 'pending')` (missing `'in_progress'`).
   - Deduplication bypass in `resend.rs` omitted `'in_progress'`.
4. **Missing Scheduler Signal & 30s Idle Sleep**:
   Restoring rows into SQLite did not send a reactive wakeup. The queue worker was stuck in a 30-second idle backoff sleep.
5. **FIFO Queue Inversion**:
   `pick_latest_prompt` used `.max_by(updated_at)` (LIFO), and `tree_format.rs` used `ORDER BY updated_at DESC`.

### 3. Architecture & Invariant Breach
- **5-Stage Pipeline Breach**: `Snapshot -> Kill -> Write Credentials -> Launch -> Restore`. Stage 5 failed-fast because of the mismatched SQL query argument.
- **Strict FIFO Ordering Breach**: Violating `ORDER BY created_at ASC, id ASC`.

### 4. Surgical Safeguards & Fixes
- `switch.rs:674-687`: Replaced `list_prompt_backups` with scoped count checks:
  `count_unrestored_prompts_for_instance(Some(&target_inst_id))` and `count_unrestored_prompts_for_other_instances`.
- `system_impl.rs:96-115`: Captured backup return count and verified both `repo_db` and `backup_prompts_db` counts.
- `backup.rs:400`: Included `OR instance_id = '' OR instance_id IS NULL` for the default instance.
- `resend.rs:77, 180` & `queue.rs:43, 154`: Added `'in_progress'` to SQL queries, deduplication bypass, and queue counts.
- `dispatch.rs:393`: Refactored to `pick_first_queued_prompt` with `min_by(created_at ASC, id ASC)`.
- `workspace.rs:210` & `post_switch_resume.rs:430`: Added immediate `notify_prompt_queue()` signals and async dispatch probes.
- `discovery.rs:76`: Removed `lower.starts_with("you are ")` false-positive subagent filter.

## Verification Evidence & Unit Tests
1. `src-tauri/src/modules/backup_prompts_db/tests.rs`:
   - `test_has_unrestored_backups_for_target_instance`: Multi-instance count scoping gate.
   - `test_prompt_status_after_restore_discoverability`: Verifies all restored outputs are recognized by the scheduler.
2. `src-tauri/src/modules/repo_db/tests.rs`:
   - `test_count_backed_up_prompts_default_includes_null_and_empty`: Verifies default alias matching.
   - `test_strict_fifo_queue_ordering`: Proves strict FIFO queue candidate selection (`created_at ASC, id ASC`).

Shipped in v4.271.0.

# Issue 64: Post-switch running prompt restore failure & status disconnect

Status: FIXED in v4.271.0 (commit `c36538824`).
Raised: 2026-10-27
Supersedes the partial and historical hypotheses in [50](./50-instance-switch-loses-running-prompt-rca.md), [57](./57-switch-marks-prompt-restored-before-ide-ready-rca.md), and [61](./61-running-prompt-resumption-and-conversation-continuity-rca.md).
Spec: [02-spec/21-app/230-running-prompt-restore-after-account-switch/01-architecture-spec.md](../../02-spec/21-app/230-running-prompt-restore-after-account-switch/01-architecture-spec.md)
RCA: [.ai-memory/rca/21-switch-restore-gate-parameter-mismatch-and-in-progress-status.md](../rca/21-switch-restore-gate-parameter-mismatch-and-in-progress-status.md)

## 1. Symptom
Across more than 50 user prompts and multiple release cycles, running prompts were consistently lost or failed to re-execute after an account switch (both manual switches and automated quota auto-switches).
Log signature:
`"[RepoDB] Single-prompt resume: no backed-up prompt for instance 'default'; nothing to dispatch"`

## 2. Trigger
Any manual account switch or background quota rotation on an instance with an active prompt.

## 3. Root Cause
1. **Critical Parameter Mismatch in `src-tauri/src/modules/instance/switch.rs:676`**:
   The post-switch restore check called:
   `crate::modules::backup_prompts_db::list_prompt_backups(Some(&target_inst_id), None)`.
   However, `list_prompt_backups` accepts `batch_id: Option<&str>` as its first parameter and filters `WHERE backup_batch_id = ?1`. Passing `&target_inst_id` (e.g. `"default"`) yielded 0 rows every time, causing `has_unrestored_backups` to evaluate to `false` and skipping prompt restore entirely.
2. **Discarded Backup Count in `DesktopIntegration::on_account_switch` (`system_impl.rs:105`)**:
   Manual switch discarded the `(backed_up_count, _)` tuple returned by `backup_active_running_prompts_for_instance` and evaluated `needs_reinject` solely using `repo_db::count_backed_up_prompts("default")`.
3. **Missing Empty String and NULL in Default Scope (`backup.rs:400`)**:
   Un-namespaced default prompts had `instance_id = ''` or `NULL`, returning 0 when counting default prompts.
4. **Restored Status Omission (`'in_progress'`)**:
   `prompt_status_after_restore` mapped restored running prompts to `'in_progress'`. But `resend_single_pass` and `check_and_dispatch_enqueued_prompts` filtered only `status IN ('backed_up', 'queued', 'pending')`, making restored prompts invisible to queue workers.
5. **No Reactive Queue Signal**:
   After re-injecting prompts, `notify_prompt_queue()` was not called, stranding workers in a 30s idle sleep.
6. **LIFO Selection Breach**:
   `pick_latest_prompt` used `.max_by(updated_at)` and `tree_format.rs` sorted by `updated_at DESC`, inverting FIFO queue ordering.

## 4. Fix
1. `switch.rs`: Direct count check via `count_unrestored_prompts_for_instance(Some(&target_inst_id))` and `count_unrestored_prompts_for_other_instances`.
2. `system_impl.rs`: Captures `backed_up_count` and checks both `repo_db` and `backup_prompts_db`.
3. `backup.rs`: Included `''` and `NULL` in default instance count queries.
4. `resend.rs` & `queue.rs`: Added `'in_progress'` to SQL queries, duplicate caches, and queue counts.
5. `dispatch.rs`: Implemented `pick_first_queued_prompt` with `min_by(created_at ASC, id ASC)`.
6. `workspace.rs` & `post_switch_resume.rs`: Injected `notify_prompt_queue()` signals and async dispatch probes.

## 5. Prevention
- Do not reuse `list_prompt_backups` for instance-scoped checks; use `count_unrestored_prompts_for_instance`.
- Enforce strict FIFO `ORDER BY created_at ASC, id ASC`.
- Covered by unit tests in `backup_prompts_db/tests.rs` and `repo_db/tests.rs`.

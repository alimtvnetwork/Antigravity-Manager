# Consolidated Plan: Auto-Switch Prompt Backup, Resumption, and Reinjection Prevention

> **Canonical Specification:** [02-spec/21-app/51-auto-switch-prompt-backup-resumption-and-deduplication.md](../../../02-spec/21-app/51-auto-switch-prompt-backup-resumption-and-deduplication.md)  
> **Root Cause Analysis:** [02-spec/22-app-issues/15-auto-switch-prompt-resumption-stuck-and-reinjection-rca.md](../../../02-spec/22-app-issues/15-auto-switch-prompt-resumption-stuck-and-reinjection-rca.md)  
> **Status:** Completed  
> **Loops/Steps Taken:** 2 Continuous Execution Steps  

---

## 1. Main Task Genesis & Verbatim Requirement

The user commanded:
```
the auto swtich still cannot do the resume the last running prompts, the prompts remains stuck, can you please fix it by backing up before switching and restore to run ? can you please and also make sure same prompt is not reinjected again and again pelase
```

---

## 2. Consolidated Subtask Execution Summary

### Subtask 01: Split DB Backup & Restore Engine Hardening
- **Target Files**: `src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/repo_db.rs`
- **Accomplishments**:
  - In `backup_active_running_prompts`: Prioritized live discovered Antigravity conversations (`discover_running_prompts_from_antigravity`). Filtered out `dispatched` prompts and any records older than 2 hours.
  - In `prompt_backups` insertion: Added check against recently restored records (`is_restored = 1 AND restored_at >= freshness_cutoff`) so old restored prompts are not repeatedly re-inserted into the backup queue unless actively running in Antigravity.
  - In `restore_running_prompts`: Reset the in-memory `DISPATCHED_PROMPTS_CACHE` via `repo_db::reset_dispatched_prompts_cache()`, allowing newly restored prompts to execute post-switch without being locked out. Ensured `resend_all_running_commands` is always called even if the backup batch table had no unrestored records.

### Subtask 02: Switch Lifecycle Consolidation and Reinjection Prevention
- **Target Files**: `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/repo_db.rs`
- **Accomplishments**:
  - In `instance::switch_account_to_instance`: Ordered pre-switch backup (`backup_running_prompts` followed by `backup_active_running_prompts`) before killing IDE processes, and triggered `restore_running_prompts` immediately upon relaunching the instance.
  - In `auto_switcher::execute_profile_rotation_with_context`: Eliminated duplicate calls to `backup_active_running_prompts`, `restore_running_prompts`, and `resend_all_running_commands`, centralizing full backup & resumption authority in `switch_account_to_instance`.
  - In `repo_db::backup_running_prompts`: Added Step 0 retirement of stale `running` prompts older than 2 hours to `dispatched`, and transitioned in-flight `running` prompts to `backed_up`.
  - In `repo_db::resend_all_running_commands`: Expanded query scope to include `'running'`, enforced latest-per-workspace dispatch, updated status to `'dispatched'`, and cached signatures to prevent repetitive reinjections in subsequent monitor ticks.
  - In `repo_db::auto_resume_recent_prompts`: Added `spawn_prompt_via_agy` call for auto-resumed prompts.

---

## 3. Verified Outcomes & Quality Gates
- `agm running-prompts backup` secured active in-flight prompt as #1 in split SQLite DB.
- `agm running-prompts restore` restored and enqueued active prompts, and subsequent immediate runs confirmed: `No unrestored prompts found in backup database` (zero reinjections).
- Unit tests: 7/7 passed in `repo_db`, 1/1 passed in `backup_prompts_db`.
- Formatting & clippy pre-flight checks: clean.

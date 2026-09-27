# Root Cause Analysis: Auto-Switch Prompt Resumption Failure & Prompt Reinjection Loop

## 1. Problem Classification & Symptom
- **Component**: Auto-Profile Switcher (`auto_switcher.rs`), Instance Lifecycle (`instance.rs`), Split Prompts DB (`backup_prompts_db.rs`), and Repository State Engine (`repo_db.rs`).
- **Symptom**:
  1. When auto-switching accounts, in-flight user prompts remained permanently stuck in the Antigravity IDE without resuming execution.
  2. Prompts were repeatedly reinjected into execution queues across successive monitor cycles and ticks.
- **Severity**: HIGH (Blocks automated unattended task continuity; floods processes with duplicate invocations).

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 In-Flight Status Misalignment (`status = 'running'`)
Before switching instances, active prompts running in the Antigravity IDE were saved in `active_prompts` with `status = 'running'`. However, `resend_all_running_commands` specifically filtered queries to `WHERE status IN ('backed_up', 'queued', 'pending')`. As a result, interrupted running prompts were completely omitted from the resumption dispatch queue and left indefinitely stuck.

### 2.2 Stale Historical Candidate Ingestion & Broken Backup Deduplication
In `backup_active_running_prompts`, candidate prompts were accumulated from all records in `active_prompts` where status was `running`, `queued`, `dispatched`, or `backed_up`. This dragged in ancient test prompts and previously completed records. Furthermore, the deduplication check in `prompt_backups` only queried `WHERE ... AND is_restored = 0`. Once a prompt was restored and marked `is_restored = 1`, subsequent backup calls re-inserted the identical prompt as a brand new unrestored record, causing an endless reinjection loop.

### 2.3 Permanent In-Memory Cache Lockout
`get_dispatched_prompts_cache()` stored signatures in an unbounded global mutex (`DISPATCHED_PROMPTS_CACHE`). When prompts were backed up and subsequently restored post-switch, `resend_all_running_commands` checked `already_dispatched = cache.contains(&sig)`. Because the prompt's signature had already been cached before the switch, `seen` evaluated to `true`, logging `Skipping duplicate dispatch` and aborting `agy` execution.

### 2.4 Redundant Dual Backup & Restore Cycles
Both `auto_switcher.rs` (in `execute_profile_rotation_with_context`) and `instance.rs` (in `switch_account_to_instance`) called `backup_active_running_prompts` and `restore_running_prompts` back-to-back. This caused duplicate batches in `backup_batches`, redundant database writes, and race conditions during prompt dispatch.

---

## 3. Remediation Strategy

1. **In-Flight State Transition & Comprehensive Query Scope**:
   - In `repo_db::backup_running_prompts`, Step 0 executes `UPDATE active_prompts SET status = 'backed_up' WHERE status = 'running'` before instance shutdown.
   - In `repo_db::resend_all_running_commands`, query scope expanded to include `status IN ('backed_up', 'queued', 'pending', 'running')`.

2. **Strict Recency & Freshness Filtering in Split DB Backup**:
   - In `backup_prompts_db::backup_active_running_prompts`, prioritize live Antigravity conversations (`discover_running_prompts_from_antigravity`).
   - For `active_prompts`, exclude `dispatched` records and require `updated_at > now - 7200` to eliminate stale test artifact pollution.
   - Deduplicate in `prompt_backups` against both unrestored and recently restored records so identical prompt text is not re-queued.

3. **Restoration Dispatch Cache Bypass**:
   - During post-switch resumption, explicitly permit restored prompts to execute via `spawn_prompt_via_agy` even if previously cached, updating their persistent DB status to `'dispatched'` and refreshing the cache with new timestamps.

4. **Single-Point Switch Orchestration**:
   - Centralize pre-switch backup and post-switch restoration strictly within `instance::switch_account_to_instance`, eliminating duplicate calls in `auto_switcher.rs`.

---

## 4. Verification & Quality Gates
- **Unit Tests**:
  - `modules::repo_db::tests::test_extract_clean_user_prompt`: Passed.
  - `modules::repo_db::tests::test_resend_deduplication_and_reinjection_prevention`: Passed.
  - `modules::repo_db::tests::test_backup_conflict_resolution_no_duplicates`: Passed.
  - `modules::backup_prompts_db::tests::test_backup_db_lifecycle`: Passed.
- **Pre-flight Linters**:
  - `cargo fmt -- --check`: Clean.
  - `cargo clippy --bin agm --lib`: Clean.

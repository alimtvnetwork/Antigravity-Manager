# Subtask 01: Backend E9001 Root-Cause Fix & Loop Throttling Hygiene

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`
**Owner:** Worker 01
**Status:** In Progress

---

## 1. Owned Files
- `src-tauri/src/modules/auto_switcher/instances.rs`
- `src-tauri/src/modules/auto_switcher/manual.rs`
- `src-tauri/src/modules/auto_switcher/selection.rs`
- `src-tauri/src/modules/repo_db/queue.rs`
- `src-tauri/src/modules/account/quota_fetch.rs`
- `src-tauri/src/modules/supabase_sync/fleet.rs`

---

## 2. Implementation Steps
1. **`src-tauri/src/modules/auto_switcher/instances.rs`**:
   - In `is_instance_binding_stale_or_exhausted`:
     - If `is_running == true`, return `false` immediately!
     - Running instances must NEVER have their accounts considered stale or stealable.
2. **`src-tauri/src/modules/auto_switcher/manual.rs`**:
   - In `trigger_manual_rotation_for_instance`:
     - Collect running sibling instances' accounts: check `inst.bound_account_id`, `inst.bound_email`, and for the default instance check `account::get_current_account_id()`. Add all to `excluded`.
3. **`src-tauri/src/modules/auto_switcher/selection.rs`**:
   - In `select_candidate_profiles`:
     - In Section 1 and Section 2, check whether candidate is bound to ANY running sibling instance on this host. If so, `continue`.
   - In `select_and_verify_next_best_profile`:
     - Add safety check before returning candidate: if candidate account is currently bound to any running sibling instance, log warning and skip to next candidate.
4. **`src-tauri/src/modules/repo_db/queue.rs`**:
   - In `check_and_dispatch_enqueued_prompts`:
     - Check `PathBuf::from(&prompt.repo_path).exists()`. If false:
       - Log warning: `workspace directory does not exist: '...' for prompt '...'; marking as failed to prevent infinite retry loop`.
       - Update prompt status to `'failed'` in `active_prompts`.
     - When `spawn_prompt_via_agy` returns `Err`:
       - Update `updated_at = now` so retry backoff applies.
5. **`src-tauri/src/modules/account/quota_fetch.rs`**:
   - In `store_l1_quota_cache`:
     - Compute adaptive TTL:
       - If quota < 20%: 120s (2m).
       - If quota < 50%: 180s (3m).
       - If quota >= 50%: 300s (5m).
6. **`src-tauri/src/modules/supabase_sync/fleet.rs`**:
   - In `start_sync_worker`:
     - On heartbeat / network error, sleep/backoff 300s (5 minutes).

---

## 3. Verification
- `cargo check --lib`
- Unit tests pass.

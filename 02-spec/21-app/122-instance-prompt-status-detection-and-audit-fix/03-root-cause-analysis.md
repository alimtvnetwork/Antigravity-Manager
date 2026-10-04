# 4-Part Root Cause Analysis: Cross-Instance Prompt Running State Contamination

## 1. Problem Description & User Symptoms

In a multi-profile environment with active instances (Sequence 1: `default` profile, PID 11628; Sequence 2: `default-copy-8159` / 8159, PID 11984):
- The `default` profile was truly executing prompts only in `Antigravity-Manager`. However, the UI card or detection logic reported `SpecBuilder` and `coding-guidelines` as running on `default`.
- The `8159` profile was truly executing prompts only in `coding-guidelines`. However, the UI card or detection logic reported `Antigravity-Manager` and `SpecBuilder` as running on `8159`.
- The running state of one instance bled into other instances, causing cards to display misleading pulsating `RUNNING` status badges.

## 2. Root Cause Analysis (4-Part RCA)

### A. Root Cause 1: Missing 15-Minute TTL in `compute_project_conversation_tree`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3740–3757)
- **Mechanism**: When scanning `conversation_summaries.db`, the query retrieved `last_time_str` (the timestamp of the most recent turn), but never validated it against the current time. If an old conversation turn had ended unexpectedly or remained in `CASCADE_RUN_STATUS_RUNNING` from hours or days ago, `is_conv_running` was evaluated as `true` simply because the host instance process was alive. Consequently, inactive historical conversations were falsely treated as active.

### B. Root Cause 2: Omission of `antigravity-cli` Directory
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `gemini_dirs_for_instance` (lines 725–730)
- **Mechanism**: `gemini_dirs_for_instance` checked only `["antigravity", "antigravity-ide"]` under `.gemini/`. CLI runs in `.gemini/antigravity-cli` were completely omitted from candidate directory discovery. This prevented CLI sessions (such as `Antigravity-Manager` running in the CLI) from being correctly identified and scoped to the instance.

### C. Root Cause 3: Unscoped Path-Only Keying in `get_live_project_execution_info`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_live_project_execution_info` (lines 2310–2320, 2364)
- **Mechanism**: The in-memory map `live_map` stored execution status keyed solely by normalized repository path (`clean_p`). When a project path (e.g. `d:/work/coding-guidelines`) had active work on instance 8159, `live_map.get("d:/work/coding-guidelines")` returned `is_running = true`. When evaluating projects belonging to `default`, the matching file path triggered `is_running = true` for `default` as well.

### D. Root Cause 4: Global Conversation Deduplication Collision
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (line 3678, 3737)
- **Mechanism**: The set `seen_tree_cids` was instantiated once globally as `HashSet<String>`. When scanning across candidate directories of multiple instances, conversation IDs were treated as globally unique without instance namespacing. This suppressed legitimate sessions and crossed instance boundaries.

### E. Root Cause 5: Permissive Fallback in `proj_is_running`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3959–3968)
- **Mechanism**: `proj_is_running` was computed as `is_inst_alive && (has_active_conv || has_active_prompt)`. Because conversations were erroneously marked `is_running = true` due to missing TTL checks, `has_active_conv` was evaluated as `true`, marking the entire project node as permanently running.

### F. Root Cause 6: Stale Disk Caching in `prompt_tree_cache`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_project_conversation_tree_cached` (lines 3520–3535)
- **Mechanism**: Cached tree results were saved under SQLite table `prompt_tree_cache`. Even if the active session ended or an instance changed state, the cached tree returned contaminated running states until the TTL expired or the cache was bypassed.

## 3. Corrective Actions & Implementation

1. **Enforce 15-Minute (900s) TTL on Conversation Turns**:
   In `compute_project_conversation_tree`, parse `last_time_str` and require `timestamp >= now - 900`. If older than 900 seconds, force `is_conv_running = false`.
2. **Add `antigravity-cli` to Candidate Directories**:
   In `gemini_dirs_for_instance`, include `"antigravity-cli"` alongside `"antigravity"` and `"antigravity-ide"`.
3. **Partition `live_map` by `(instance_id, repo_path)`**:
   Key `live_map` by composite key `(String, String)` so that projects are only matched to the instance that actually owns the execution.
4. **Namespace `seen_tree_cids` by `(owning_inst_id, cid)`**:
   Ensure deduplication is partitioned per instance.
5. **Strict Idle Supremacy**:
   Ensure `is_idle_count` (`not_fully_idle == 0`) or status containing `IDLE`, `COMPLETED`, `FAILED`, `CANCELLED` unconditionally overrides running status.
6. **Structured Audit Logging**:
   Log all gate evaluations and rationale via `log_instance_prompt_audit`.

## 4. Verification & Prevention

- Author end-to-end tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs` covering cross-instance isolation, TTL expiry, and process death.
- Future AI rule: Never evaluate liveness of background turns without validating turn recency (TTL) against host OS time and scoping strictly to the instance ID.

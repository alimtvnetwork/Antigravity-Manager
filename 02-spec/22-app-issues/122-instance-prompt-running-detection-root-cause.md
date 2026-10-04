# 4-Part RCA: Cross-Instance Prompt Running State Contamination & False Liveness Detection

| Field | Details |
| :--- | :--- |
| **Incident / Defect ID** | `122` |
| **Affected Components** | `src-tauri/src/modules/repo_db.rs`, `src/pages/Instances.tsx`, `prompt_tree_cache` |
| **Severity** | High (Erroneous running prompt reporting across instances, cross-instance state pollution) |
| **Resolution Status** | Resolved |

---

## 1. Problem Description & User Symptoms

In a multi-instance setup:
- Instance 1 (`default`): Truly executing `Antigravity-Manager`. Erroneously displayed `SpecBuilder` and `coding-guidelines` as running.
- Instance 2 (`default-copy-8159` / 8159): Truly executing `coding-guidelines`. Erroneously displayed `Antigravity-Manager` and `SpecBuilder` as running.
- Multiple cards in the UI showed pulsating running badges for projects that were completely idle in those instances.

---

## 2. Root Cause Analysis (4-Part RCA)

### A. Root Cause 1: Missing 15-Minute TTL Check in Conversation Tree Builder
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3740–3757)
- **Mechanism**: The query against `conversation_summaries.db` retrieved `last_modified_time`, but never validated whether the turn was fresh. If an ancient conversation from hours or days ago remained in `CASCADE_RUN_STATUS_RUNNING` (e.g. from an aborted session), it was treated as currently running as long as the instance process was alive.

### B. Root Cause 2: Omission of `antigravity-cli` in Gemini Candidate Directory Discovery
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `gemini_dirs_for_instance` (lines 725–730)
- **Mechanism**: Only `["antigravity", "antigravity-ide"]` were checked. CLI sessions in `.gemini/antigravity-cli` were not discovered for the default instance, preventing proper correlation.

### C. Root Cause 3: Unpartitioned Path-Only Keying in `live_map`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_live_project_execution_info` (lines 2310–2320, 2364)
- **Mechanism**: `live_map` was keyed solely by normalized repository path without `instance_id`. When a project path had active tasks on instance 8159, all instances having that same repository in their workspace storage evaluated `is_running = true`.

### D. Root Cause 4: Global Deduplication Collision in `seen_tree_cids`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (line 3678, 3737)
- **Mechanism**: `seen_tree_cids` was an unpartitioned global `HashSet<String>`. Sessions read under one instance could not be indexed or correctly assigned to another instance.

### E. Root Cause 5: Inadequate Idle Supremacy Over-Ride
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3740–3755)
- **Mechanism**: Idle statuses (`IDLE`, `COMPLETED`, `FAILED`, `CANCELLED`) and turn counters (`not_fully_idle == 0`) were not strictly prioritized before liveness fallback checks.

### F. Root Cause 6: Stale Disk Caching in `prompt_tree_cache`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_project_conversation_tree_cached` (lines 3520–3535)
- **Mechanism**: SQLite table `prompt_tree_cache` stored stale tree nodes with `is_running: true`, which continued to be served across subsequent UI refresh calls.

---

## 3. Corrective Actions & Implementation

1. **Enforce 15-Minute (900s) TTL on Conversation Turns**:
   In `compute_project_conversation_tree`, parse `last_time_str` and require `timestamp >= now - 900`. If older than 900 seconds, unconditionally set `is_conv_running = false`.
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

---

## 4. Verification & Prevention

- Comprehensive integration tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs` validating exact instance-project running isolation.
- Architectural invariant: Background turn liveness MUST ALWAYS be gated by:
  1. Host process OS PID liveness.
  2. Strict turn timestamp TTL (< 900s).
  3. Strict instance ID matching.
  4. Explicit idle supremacy.

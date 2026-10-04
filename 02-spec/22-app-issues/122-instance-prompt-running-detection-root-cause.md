# 4-Part RCA: Cross-Instance Prompt Running State Contamination & False Liveness Detection

| Field | Details |
| :--- | :--- |
| **Incident / Defect ID** | `122` |
| **Affected Components** | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/logger.rs`, `prompt_tree_cache`, `src/pages/Instances.tsx` |
| **Severity** | High (Erroneous running prompt reporting across instances, cross-instance state pollution) |
| **Resolution Status** | Resolved |
| **Associated Spec** | [02-spec/21-app/122-instance-prompt-status-detection-and-audit-fix/03-root-cause-analysis.md](../21-app/122-instance-prompt-status-detection-and-audit-fix/03-root-cause-analysis.md) |

---

## 1. Problem Description & User Symptoms

In a multi-instance desktop deployment:
- **Instance 1 (`default`)**:
  - PID: 11628 (`Antigravity.exe`)
  - Truly executing: `Antigravity-Manager`
  - Defect symptom: Erroneously displayed `SpecBuilder` and `coding-guidelines` as running.
- **Instance 2 (`default-copy-8159` / 8159)**:
  - PID: 11984 (`Antigravity-default-copy-8159.exe`)
  - Truly executing: `coding-guidelines`
  - Defect symptom: Erroneously displayed `Antigravity-Manager` and `SpecBuilder` as running.
- **Visual Impact**:
  Project cards in the UI showed pulsating running badges for completely idle or dormant projects across instances, confusing users and violating cross-instance process isolation.

---

## 2. Root Cause Analysis (4-Part RCA)

### Root Cause 1: Missing 15-Minute TTL Check in Conversation Tree Builder
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3740–3757)
- **Mechanism**: The query against `conversation_summaries.db` retrieved `last_time_str`, but never validated whether the turn was fresh. If an ancient conversation from hours or days ago remained in `CASCADE_RUN_STATUS_RUNNING` (e.g. from an aborted session), it was treated as currently running as long as the instance process was alive.

### Root Cause 2: Omission of `antigravity-cli` in Gemini Candidate Directory Discovery
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `gemini_dirs_for_instance` (lines 725–730)
- **Mechanism**: Only `["antigravity", "antigravity-ide"]` were checked under `.gemini/`. Standalone CLI sessions in `.gemini/antigravity-cli` were completely omitted from candidate directory discovery, preventing CLI prompt status correlation.

### Root Cause 3: Timestamp Format Parsing Mismatch in Gate 4 & SQLite Records
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project` (lines 1745–1761)
- **Mechanism**: Antigravity writes timestamps across different formats (RFC 3339 with timezone, SQLite local time `YYYY-MM-DD HH:MM:SS`, ISO `YYYY-MM-DDTHH:MM:SS`). Without a resilient multi-format parser, valid timestamps were either rejected or bypassed into permissive legacy fallback paths.

### Root Cause 4: Unpartitioned Path-Only Keying in `live_map`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_live_project_execution_info` (lines 2310–2320, 2364)
- **Mechanism**: `live_map` was keyed solely by normalized repository path without `instance_id`. When a project path had active tasks on instance 8159, all instances having that same repository in their workspace storage evaluated `is_running = true`.

### Root Cause 5: Global Conversation Deduplication Collision
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3678, 3737)
- **Mechanism**: `seen_tree_cids` was an unpartitioned global `HashSet<String>`. Sessions read under one instance could not be indexed or correctly assigned to another instance.

### Root Cause 6: Stale Disk Caching in `prompt_tree_cache`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `get_project_conversation_tree_cached` (lines 3520–3535)
- **Mechanism**: SQLite table `prompt_tree_cache` stored stale tree nodes with `is_running: true`, which continued to be served across subsequent UI refresh calls until TTL expired or the cache was explicitly bypassed.

---

## 3. Corrective Actions & Implementation

1. **Enforce 15-Minute (900s) TTL on Conversation Turns**:
   In `compute_project_conversation_tree`, parse `last_time_str` into UNIX timestamp using a multi-format parser and require `timestamp >= now - 900`. If older than 900 seconds, unconditionally set `is_conv_running = false`.
2. **Add `antigravity-cli` to Candidate Directories**:
   In `gemini_dirs_for_instance`, include `"antigravity-cli"` alongside `"antigravity"` and `"antigravity-ide"`.
3. **Partition `live_map` by `(instance_id, repo_path)`**:
   Key `live_map` by composite key `(String, String)` so that projects are only matched to the instance that actually owns the execution.
4. **Namespace `seen_tree_cids` by `(owning_inst_id, cid)`**:
   Ensure deduplication is partitioned per instance using `HashSet<(String, String)>`.
5. **Strict Idle Supremacy**:
   Ensure `is_idle_count` (`not_fully_idle == 0`) or status containing `IDLE`, `COMPLETED`, `FAILED`, `CANCELLED` unconditionally overrides running status across all gates.
6. **Structured Audit Logging**:
   Log all gate evaluations and rationale via `log_instance_prompt_audit`.

---

## 4. Verification & Prevention

- Comprehensive integration tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs`:
  * `test_case_ttl_stale_conversation_expiry_forces_idle`: Validates that turns older than 15 minutes are forced idle.
  * `test_case_antigravity_cli_discovery`: Verifies candidate discovery discovers `.gemini/antigravity-cli`.
  * `test_case_composite_keying_isolation_same_repo_path`: Verifies that concurrent instances sharing the same repository path maintain absolute execution isolation.
- Invariants Enforced:
  1. Real-time OS PID liveness is mandatory (Gate 0).
  2. Timestamp freshness (TTL <= 900s) is mandatory for conversation turns.
  3. Composite keying `(instance_id, repo_path)` is mandatory for all live execution maps.
  4. Explicit idle status takes unconditional supremacy over running substrings.

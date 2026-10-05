# Root Cause Analysis: Running Projects False Positives & Instance Lifecycle UX

## 1. Defect Description & Observed Failure
Users experienced:
1. **Missing Restart Action**: Instances could only be stopped or launched. To restart on the current account, users had to stop, wait, and manually launch, creating UX friction.
2. **Confusing Sync Icons**: Buttons performing process ID sync used rotating circular arrows (`RotateCw` / `RefreshCw`), making users believe they were restarting the instance.
3. **Buggy Running Projects Detection**:
   - The Default instance falsely showed `white-presentation-v1` (or other projects) as running, even though the Default instance had no such project running.
   - Ghost/empty untitled conversations were registered as active.
   - Closed IDE windows continued to report `is_running: true`.

---

## 2. Root Cause Analysis (4 Structural Failure Mechanisms)

### RCA-01: Default Instance Query Fallback in `repo_db.rs`
- **Location**: `src-tauri/src/modules/repo_db.rs` around lines 1627, 1739, 1982, 3293, 3340
- **Flaw**: SQL queries for the default instance used:
  `(?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = ''))`
- **Impact**: Any orphaned project record or legacy prompt with `instance_id IS NULL` was greedily attributed to the Default instance, making it falsely report third-party projects as running.

### RCA-02: 0-Word Untitled Ghost Turn Contamination
- **Flaw**: When Antigravity IDE launches, it frequently registers a blank scratch session. These records have 0 prompt words and title "Untitled Conversation". Previous logic only checked `title.to_lowercase().starts_with("untitled")` without strictly gating on `prompt_word_count == 0` during active running evaluation.
- **Impact**: Blank conversations triggered false running states.

### RCA-03: Stale Cache in `prompt_tree_cache` Table
- **Flaw**: The SQLite cache table `prompt_tree_cache` cached serialized JSON representations with a TTL that did not invalidate when an IDE process exited or when a prompt completed.
- **Impact**: Users reopening the UI saw stale cached running indicators.

### RCA-04: Ambiguous Icon Semantics in Frontend Action Strips
- **Flaw**: `RotateCw` was reused for both "Sync PID" and "Fast Forward", visually mimicking the universal restart symbol.
- **Impact**: Visual confusion between non-destructive metadata sync and destructive process restarts.

---

## 3. Remediation & Preventive Measures
1. **Surgical SQL Hardening**: Remove `instance_id IS NULL OR instance_id = ''` from all Default instance queries in `repo_db.rs`. Require explicit instance binding.
2. **Heartbeat & Process Guard**: Require both `is_inst_alive == true` AND `(now - last_turn_ts <= 120)` for a project to be considered running.
3. **Ghost Conversation Filter**: Unconditionally discard 0-word untitled conversations from running counters.
4. **Distinct Icon Semantics**: Use `RotateCcw` exclusively for Restart, and `Cpu` / `ArrowLeftRight` for Sync operations.
5. **Atomic Split Button**: Combine Stop and Restart into a single contiguous segmented capsule when running.

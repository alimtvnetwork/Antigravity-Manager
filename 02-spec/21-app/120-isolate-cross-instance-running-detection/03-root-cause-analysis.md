# Root Cause Analysis: Cross-Instance Running Detection Isolation & False Running State Bleed

> **Document:** `03-root-cause-analysis.md`  
> **Task ID:** `120-isolate-cross-instance-running-detection`  
> **Status:** APPROVED & ARCHITECTED  
> **Domain:** Multi-Instance Liveness & Process Isolation  
> **Author:** Worker 01 (Architecture & Root Cause Analysis Author)  
> **Date:** October 2026  

---

## 1. Executive Summary & Problem Classification

### 1.1 The Multi-Instance Running Bleed Phenomenon
In modern Antigravity Manager (AGM) setups, users run multiple independent instances:
- **Default Profile (`default`)**: Running the default user configuration in `~/.gemini` and default IDE workspace storage.
- **Cloned Secondary Profile (`default-copy-8159` / shorthand `8159`)**: Running an isolated sandbox environment with independent home and data directories.

When users run workloads simultaneously across these instances:
1. **Default Profile** was actively executing **ONLY** `Antigravity-Manager`. `coding-guidelines` and other repositories were idle.
2. **Instance 8159** was actively executing **ONLY** `coding-guidelines`. `Antigravity-Manager` was idle.

### 1.2 Observed Failure Symptoms
Despite complete physical process and filesystem isolation between the two instances, the AGM UI displayed severe cross-instance execution bleeding:
- **Default Profile Card**: Incorrectly displayed `coding-guidelines` as actively running with a green/cyan pulse badge.
- **Instance 8159 Card**: Incorrectly displayed `Antigravity-Manager` as actively running.
- **Workspace Storage Cross-Pollution**: Any historical workspace ever opened in either instance could flip to `is_running = true` if an active prompt was running in any other instance on the system.
- **Vanishing Workspaces**: Workspaces in `default` would intermittently disappear from the instance view when `8159` was scanned.

This 4-part Root Cause Analysis details the exact architectural and code defects responsible for this behavior and establishes non-negotiable invariants to prevent regressions.

---

## 2. Root Cause Analysis: The 6 Core Flaws

Rigorous analysis of `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`, and related subsystems revealed six compounding root causes:

---

### Flaw 1: Primary Key Collision in `running_projects` SQLite Table
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `detect_running_projects` (lines 515–586)
- **Flawed Code**:
  ```rust
  let project_id = format!(
      "{}-{}",
      repo_name.to_lowercase(),
      entry.file_name().to_string_lossy()
  );

  // SQLite upsert
  conn.execute(
      "INSERT INTO running_projects 
       (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
       ON CONFLICT(id) DO UPDATE SET
          instance_id = excluded.instance_id,
          repo_name = excluded.repo_name,
          repo_path = excluded.repo_path,
          workspace_storage_path = excluded.workspace_storage_path,
          is_running = excluded.is_running,
          last_detected_at = excluded.last_detected_at,
          updated_at = excluded.updated_at",
      // ...
  );
  ```
- **Why It Is Flawed**:
  The `running_projects` table defines `id TEXT PRIMARY KEY`. The workspace storage folder names under `User/workspaceStorage/<hash>` are derived from the workspace path by VSCode/Antigravity. When an instance is cloned (e.g., `default` cloned to `default-copy-8159`), all workspace folder hashes are copied verbatim.
  Because `project_id` was constructed as `format!("{}-{}", repo_name.to_lowercase(), hash)`, both instances generated identical `id` strings (e.g., `antigravity-manager-d58c5517`).
  When `detect_running_projects("default-copy-8159")` ran, `ON CONFLICT(id) DO UPDATE` overwrote the existing row for `default`, changing `instance_id` to `"default-copy-8159"`. When `detect_running_projects("default")` ran next, it overwrote it back. Consequently:
  - Whichever instance scanned last "stole" ownership of the project row.
  - Queries filtering by `instance_id` dropped the project from the other instance.
  - The `is_running` state from one instance directly overwrote the running state of the other.
- **Correct Pattern**:
  Prefix the primary key with `target_id`:
  ```rust
  let project_id = format!(
      "{}:{}-{}",
      target_id,
      repo_name.to_lowercase(),
      entry.file_name().to_string_lossy()
  );
  ```
  Every row is partitioned strictly per instance.

---

### Flaw 2: Cross-Instance CID Deduplication in `compute_project_conversation_tree`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` (lines 3611, 3670)
- **Flawed Code**:
  ```rust
  let mut seen_tree_cids = std::collections::HashSet::new();

  for (owning_inst_id, base) in &candidate_dirs {
      // ...
      for item in rows.flatten() {
          let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_time_str) = item;
          if !seen_tree_cids.insert(cid.clone()) {
              continue; // FLAW: CID deduplication across instance boundaries
          }
  ```
- **Why It Is Flawed**:
  `seen_tree_cids` was declared as `HashSet<String>`. When `target_instance` is None (or when scanning all candidate directories), both `default` and cloned instances (e.g., `default-copy-8159`) contain conversation databases with identical historical conversation IDs (`cid`).
  If `default` was processed first, every CID in its database was inserted into `seen_tree_cids`. When the iterator proceeded to `default-copy-8159`, `!seen_tree_cids.insert(cid.clone())` returned `true` (already exists) and skipped every historical conversation in `default-copy-8159`. This masked conversations in secondary profiles or attributed conversations across instances.
- **Correct Pattern**:
  Scope the deduplication set by a composite key `(instance_id, cid)`:
  ```rust
  let mut seen_tree_cids: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
  // ...
  let cid_key = (norm_owning_inst.clone(), cid.clone());
  if !seen_tree_cids.insert(cid_key) {
      continue;
  }
  ```

---

### Flaw 3: Loose Substring Path Matching in `is_prompt_running_for_project` (Step 4)
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project` (lines 1722–1725)
- **Flawed Code**:
  ```rust
  let clean_p = normalize_path_for_compare(&decode_uri_to_path(&u));
  let has_target = !clean_target.is_empty();
  let matches_path = clean_p == clean_target
      || clean_p.contains(&clean_target)
      || clean_target.contains(&clean_p);
  let is_target_matched = has_target && matches_path;
  ```
- **Why It Is Flawed**:
  `clean_p.contains(&clean_target) || clean_target.contains(&clean_p)` performed a bidirectional substring check.
  1. If `clean_target` was `d:/work/Antigravity-Manager` and another workspace was `d:/work/Antigravity-Manager-Docs`, both matched.
  2. If `clean_target` was a parent folder or drive letter, every single conversation summary on the system matched.
  3. When an active conversation in `conversation_summaries.db` belonged to a different directory sharing a prefix or substring, it falsely reported the target project as running.
- **Correct Pattern**:
  Enforce strict normalized path equality:
  ```rust
  let matches_path = clean_p == clean_target;
  let is_target_matched = has_target && matches_path;
  ```
  Both paths are already canonicalized and stripped of case/trailing slash discrepancies. Exact equality prevents false matches.

---

### Flaw 4: Unbounded Turn Timestamp & Inverted OR Logic in `conversation_summaries.db`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project` (Step 4, lines 1683–1713)
- **Flawed Code**:
  ```rust
  let (status, not_fully_idle, ws_uris_opt, _last_time_str) = item;

  // Strict idle supremacy rule:
  let is_idle_count = not_fully_idle == 0;
  let has_idle_status = status.contains("IDLE")
      || status.contains("COMPLETED")
      || status.contains("FAILED")
      || status.contains("CANCELLED");
  let is_explicit_idle = is_idle_count || has_idle_status;

  let is_conv_running = if is_explicit_idle {
      false
  } else {
      not_fully_idle != 0 || status.contains("RUNNING") // FLAW: Unchecked timestamp & OR logic
  };
  ```
- **Why It Is Flawed**:
  1. `_last_time_str` was discarded. If an IDE process crashed or closed mid-turn days or weeks earlier while `status` was `"RUNNING"`, the record persisted at the top of `conversation_summaries.db`. Without a recency cutoff, that dead session caused `is_prompt_running_for_project` to return `true` permanently whenever the instance process was running.
  2. The fallback branch used `not_fully_idle != 0 || status.contains("RUNNING")`. If `status` was not yet updated to an idle string but `not_fully_idle == 0`, or vice-versa, the OR branch evaluated to `true`.
- **Correct Pattern**:
  1. Parse `last_time_str` as RFC3339 and enforce a 15-minute Time-To-Live (900 seconds):
     ```rust
     let is_recent = chrono::DateTime::parse_from_rfc3339(&last_time_str)
         .map(|dt| dt.timestamp() >= now - 900)
         .unwrap_or(false);
     if !is_recent {
         continue;
     }
     ```
  2. Require BOTH active turns AND running status string:
     ```rust
     let is_conv_running = if is_explicit_idle {
         false
     } else {
         not_fully_idle > 0 && status.contains("RUNNING")
     };
     ```

---

### Flaw 5: Worker Key Matching Normalization in `get_active_agy_workers`
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project` (Step 2, lines 1580–1590)
- **Flawed Code**:
  ```rust
  let expected_prefix = format!("{}:", norm_inst);

  for (key, &pid) in workers.iter() {
      let has_prefix = key.starts_with(&expected_prefix);
      let is_default_match = norm_inst == "default" && !key.contains(':');
      let matches_inst = norm_inst == "all" || has_prefix || is_default_match;
      let matches_proj = key.contains(project_id); // FLAW: Loose substring matching
  ```
- **Why It Is Flawed**:
  1. `key.contains(project_id)` was a raw string substring check. If a worker was registered for `default:d:\work\Antigravity-Manager-Docs`, querying `Antigravity-Manager` matched.
  2. Path separators in `key` (often backslashes `\` on Windows) clashed with forward slashes `/` in `project_id`.
  3. If `norm_inst` was passed as `"8159"`, but the worker was registered with key `"default-copy-8159:..."`, prefix matching failed to find the active worker.
- **Correct Pattern**:
  Split the worker key by `:`, normalize both the instance and repository path, and enforce strict normalized path comparison:
  ```rust
  if let Some((worker_inst, worker_path)) = key.split_once(':') {
      let is_inst_match = norm_inst == "all"
          || worker_inst.eq_ignore_ascii_case(norm_inst)
          || resolved_inst_id.map(|r| worker_inst.eq_ignore_ascii_case(&r)).unwrap_or(false)
          || (norm_inst == "default" && worker_inst.is_empty());
      let clean_worker_path = normalize_path_for_compare(worker_path);
      let is_path_match = clean_worker_path == clean_target;
      if is_inst_match && is_path_match {
          // verify PID alive
      }
  }
  ```

---

### Flaw 6: Suffix Matching Deficiency in `resolve_instance_id`
- **Location**: `src-tauri/src/modules/instance.rs` -> `resolve_instance_id` (lines 3837–3865)
- **Flawed Code**:
  ```rust
  // Check if numeric seq_num (e.g. "1")
  if let Ok(num) = clean.parse::<u32>() {
      if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
          return Ok(inst.id.clone());
      }
  }
  // Check exact id or name match
  if let Some(inst) = registry
      .instances
      .iter()
      .find(|i| i.id.eq_ignore_ascii_case(clean) || i.name.eq_ignore_ascii_case(clean))
  {
      return Ok(inst.id.clone());
  }
  Err(format!("Instance '{}' not found", clean))
  ```
- **Why It Is Flawed**:
  When users or CLI commands reference a cloned instance by its unique numeric suffix (e.g. `"8159"` for instance `"default-copy-8159"`):
  1. `clean.parse::<u32>()` parsed `8159`, but looked for `seq_num == Some(8159)`. Cloned instances have sequential numbers like `2` or `3`, not `8159`.
  2. The exact ID/name check failed because `"default-copy-8159"` does not equal `"8159"`.
  3. The function failed with `Instance '8159' not found`.
  4. Calls to `is_prompt_running_for_project(path, "8159")` could not resolve the instance data directory or running PID, causing fallback failures and state desynchronization.
- **Correct Pattern**:
  Add suffix matching for instance IDs and names:
  ```rust
  if let Some(inst) = registry.instances.iter().find(|i| {
      i.id.ends_with(&format!("-{}", clean))
          || i.id.eq_ignore_ascii_case(clean)
          || i.name.ends_with(&format!("-{}", clean))
          || i.name.eq_ignore_ascii_case(clean)
  }) {
      return Ok(inst.id.clone());
  }
  ```

---

## 3. Ground Truth Invariants Matrix

The physical ground truth of multi-instance running status must always satisfy this matrix:

| Instance ID | Target Workspace | True Runtime State | Required Backend `is_running` | Required Frontend Badge | Rationale |
|---|---|---|---|---|---|
| `default` | `Antigravity-Manager` | In-Flight Prompt Running | `true` | `[RUNNING]` (cyan/pulse) | Active worker / live turn in `default` |
| `default` | `SpecBuilder` | Idle (No Active Task) | `false` | None / Idle | Process alive, zero active tasks in `default` |
| `default` | `coding-guidelines` | Idle (No Active Task) | `false` | None / Idle | Process alive, zero active tasks in `default` |
| `default-copy-8159` (`8159`) | `coding-guidelines` | In-Flight Prompt Running | `true` | `[RUNNING]` (cyan/pulse) | Active worker / live turn in `8159` |
| `default-copy-8159` (`8159`) | `Antigravity-Manager` | Idle (No Active Task) | `false` | None / Idle | Process alive, zero active tasks in `8159` |
| `default-copy-8159` (`8159`) | `SpecBuilder` | Idle (No Active Task) | `false` | None / Idle | Process alive, zero active tasks in `8159` |

---

## 4. Recommendations & Guardrails for Future AI Developers

To prevent any future AI assistant or human developer from re-introducing these bugs, adhere to the following seven non-negotiable rules:

1. **NEVER use bare repo names or hashes as SQLite Primary Keys**: Always namespace `running_projects.id` with the owning `target_id` prefix (`{target_id}:{repo_name}-{hash}`).
2. **NEVER deduplicate across instances with single-string Sets**: Any CID cache or deduplication set in multi-instance aggregation must use a composite key `(instance_id, cid)`.
3. **NEVER use `contains()` for filesystem path comparisons**: Filesystem paths must be canonicalized and checked via strict equality (`clean_p == clean_target`). Substring matching causes directory bleed.
4. **ALWAYS enforce TTL on external SQLite session tables**: Never trust historical status flags in `conversation_summaries.db` without validating that `last_modified_time` is within the 15-minute active window (`<= 900s`).
5. **ALWAYS enforce Idle Supremacy with Strict AND**: A session is only running if it is recent, has `not_fully_idle > 0`, and `status.contains("RUNNING")`. If `not_fully_idle == 0` or status contains `IDLE`/`COMPLETED`, it is strictly idle.
6. **ALWAYS normalize and split worker map keys**: Subagent worker keys must be split into `(instance_id, path)`, with each component compared individually under normalization.
7. **ALWAYS emit structured audit logs (`log_instance_prompt_audit`)**: Never silently determine liveness. Every decision must be emitted to the log with instance, project path, active task count, and clear rationale.

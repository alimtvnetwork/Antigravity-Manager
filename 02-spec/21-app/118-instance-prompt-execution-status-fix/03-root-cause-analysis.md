# Root Cause Analysis: Cross-Instance Prompt Running Detection & False Running State Bleed

## Executive Summary
This document provides a comprehensive 4-part Root Cause Analysis (RCA) explaining why the prompt running detection on Antigravity instances produced false-positive `[RUNNING]` badges and cross-instance state pollution. This reference is preserved so that any future AI or developer will understand why the historical approach was flawed and why the architectural remediation is designed as it is.

---

## 1. Problem Classification & Observation
In multi-instance environments:
- **Default Profile (Sequence 1)**: Actively running ONLY `Antigravity-Manager`. Projects `SpecBuilder` and `coding-guidelines` were idle, but displayed active `[RUNNING]` badges.
- **Instance 8159 (Sequence 2)**: Actively running ONLY `coding-guidelines`. Projects `Antigravity-Manager` and `SpecBuilder` were idle, but displayed active `[RUNNING]` badges.

Instead of each instance independently reflecting its true runtime state, active execution on one instance bled into other instances, causing idle projects to appear as actively running across profile boundaries.

---

## 2. Root Cause Analysis (The 7 Flaws)

### Flaw 1: Unscoped Subagent Worker Key Matching in Memory
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project`
- **Flawed Code**:
  ```rust
  if let Ok(workers) = get_active_agy_workers().lock() {
      for key in workers.keys() {
          if key.contains(project_id) {
              return true; // FLAW: Ignores instance_id!
          }
      }
  }
  ```
- **Why It Is Wrong**: The worker map keys are formatted as `"<instance_id>:<repo_path>"`. By checking only `key.contains(project_id)`, any worker running on instance `8159` matching `d:/work/coding-guidelines` matches when queried by `default`, and any worker on `default` running `Antigravity-Manager` matches when queried by `8159`. This creates immediate bidirectional cross-instance contamination.
- **Correct Pattern**: Must verify that the key belongs to the queried instance:
  ```rust
  let instance_prefix = format!("{}:", instance_id);
  if (key.starts_with(&instance_prefix) || (instance_id == "default" && !key.contains(':'))) && key.contains(project_id) {
      return true;
  }
  ```

### Flaw 2: Frontend Raw Status Override Over Backend Boolean
- **Location**: `src/pages/Instances.tsx` (`isProjRunning`, `hasActiveTask`, `fetchRunningTasks`)
- **Flawed Code**:
  ```typescript
  const isProjRunning = Boolean(inst.is_running) && Boolean(
      proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING')
  );
  ```
- **Why It Is Wrong**: The backend sets `c.is_running = false` when an instance or project is not actively executing, but keeps `c.status` populated with the raw historical string from the SQLite database. If a past conversation had `status = "RUNNING"`, the frontend's `|| c.status === 'RUNNING'` completely bypassed the backend's definitive `is_running = false`. Because the instance process was alive (`inst.is_running === true`), the project falsely lit up with a pulsing green/cyan `[RUNNING]` badge.
- **Correct Pattern**: Frontend must derive liveness strictly from `Boolean(proj.is_running) || Boolean(proj.conversations?.some((c) => Boolean(c.is_running)))`. Never use raw historical status strings to override boolean liveness.

### Flaw 3: Inverted OR Logic on Conversation Idle State
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree` & `is_prompt_running_for_project`
- **Flawed Code**:
  ```rust
  let is_conv_running = is_owning_inst_alive && (not_fully_idle != 0 || status.contains("RUNNING"));
  ```
- **Why It Is Wrong**: In the Antigravity IDE SQLite database, `not_fully_idle = 0` indicates the model/turn is completely idle. However, the IDE does not immediately mutate the historical string `status` (which often retains `"RUNNING"` or `"CASCADE_RUN_STATUS_RUNNING"`). Using `||` causes `status.contains("RUNNING")` to evaluate to `true` even when `not_fully_idle == 0`.
- **Correct Pattern**: Strict idle supremacy: If `not_fully_idle == 0` OR `status` contains `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, `is_conv_running` is strictly `false`.

### Flaw 4: Cloned Profiles Inherit Runtime Execution States
- **Location**: `src-tauri/src/modules/repo_db.rs` (`clone_instance_repo_rows`) & `src-tauri/src/modules/instance.rs` (`copy_gemini_trees`)
- **Flawed Code**:
  When cloning instance `default` into `8159`, `running_projects` rows were copied with `is_running = 1`, `active_prompts` were copied with `status = 'running'`, and `conversation_summaries.db` was copied with stale running records.
- **Why It Is Wrong**: A cloned profile starts as an independent dormant copy. Copying active running states means as soon as the new instance launches, it thinks it is running all projects that were running on the source instance at clone time.
- **Correct Pattern**: Cloned `running_projects` must be forced to `is_running = 0`. Cloned `active_prompts` must have status reset to `'completed'` or deleted. Cloned `conversation_summaries.db` must be sanitized (`UPDATE conversation_summaries SET not_fully_idle = 0, status = 'IDLE'`).

### Flaw 5: Queued & Backed-Up Prompts Stamped as Running
- **Location**: `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree`
- **Flawed Code**:
  ```rust
  let is_run = is_inst_alive && (ap.status == "running" || ap.status == "queued" || ap.status == "backed_up");
  ```
- **Why It Is Wrong**: A prompt that is waiting in queue or backed up is NOT actively executing. Treating queued prompts as `is_running = true` causes idle projects to display active execution badges before execution ever starts.
- **Correct Pattern**: Only `ap.status == "running"` sets `is_run = true`. Queued and backed_up prompts must have `is_running = false`.

### Flaw 6: Active Prompts Never Marked as Terminal
- **Location**: `src-tauri/src/modules/repo_db.rs`
- **Why It Is Wrong**: `active_prompts` rows were inserted with `status = 'running'`, but never updated to `completed` or `failed` upon completion. The query `SELECT COUNT(*) FROM active_prompts WHERE status = 'running'` thus accumulated stale rows that survived indefinitely.
- **Correct Pattern**: Ensure completion handlers update `active_prompts` status to `completed` or `failed`, or apply a bounded TTL check against process liveness.

### Flaw 7: Unscoped Global Conversation Tree Polling
- **Location**: `src/pages/Instances.tsx` -> `fetchRunningTasks`
- **Flawed Code**:
  ```typescript
  const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
      maxWords: 50,
      onlyRunning: false,
      force: false,
      // instanceId omitted!
  });
  ```
- **Why It Is Wrong**: Omitting `instanceId` aggregates projects and conversations across all known data directories into one unified tree. The frontend then checked if `runningTreeNodes` had any running node without strict instance scoping, cross-pollinating task indicators on instance cards.
- **Correct Pattern**: Query and evaluate conversation trees strictly scoped to `inst.config.id` or filter nodes explicitly by `node.instance_id === inst.config.id`.

---

## 3. Ground Truth Invariants Matrix

| Instance | Project | Expected State | Required Backend `is_running` | Required Frontend Badge | Rationale |
|---|---|---|---|---|---|
| **Default Profile** | `Antigravity-Manager` | **RUNNING** | `true` | `[RUNNING]` (cyan/pulse) | Active worker / process present on Default |
| **Default Profile** | `SpecBuilder` | **IDLE** | `false` | None / Idle | No worker, not_fully_idle == 0 on Default |
| **Default Profile** | `coding-guidelines` | **IDLE** | `false` | None / Idle | No worker, not_fully_idle == 0 on Default |
| **Instance 8159** | `Antigravity-Manager` | **IDLE** | `false` | None / Idle | No worker, not_fully_idle == 0 on 8159 |
| **Instance 8159** | `SpecBuilder` | **IDLE** | `false` | None / Idle | No worker, not_fully_idle == 0 on 8159 |
| **Instance 8159** | `coding-guidelines` | **RUNNING** | `true` | `[RUNNING]` (cyan/pulse) | Active worker / process present on 8159 |

---

## 4. Structured Audit Logging Architecture

To ensure any discrepancy can be diagnosed immediately from logs without guessing, every evaluation in `detect_running_projects` and `compute_project_conversation_tree` must emit a structured audit log line via `log_instance_prompt_audit`:

```text
[InstancePromptAudit] instance='8159' project='Antigravity-Manager' path='d:/work/Antigravity-Manager' is_instance_active=true is_running=false active_tasks=0 rationale='IDLE_SUPREMACY: not_fully_idle=0, no active worker for instance 8159'
[InstancePromptAudit] instance='8159' project='coding-guidelines' path='d:/work/coding-guidelines' is_instance_active=true is_running=true active_tasks=1 rationale='ACTIVE_WORKER_MATCH: worker key 8159:d:/work/coding-guidelines is active'
```

Fields:
- `instance`: exact instance ID (`default`, `8159`, etc.)
- `project`: normalized project name
- `path`: workspace file path
- `is_instance_active`: whether instance OS process is alive
- `is_running`: final decision
- `active_tasks`: number of active workers/conversations
- `rationale`: transparent reason string detailing why this state was assigned

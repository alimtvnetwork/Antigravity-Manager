# 114 Root Cause Analysis: Instance Prompt Running Detection and Multi-Instance Isolation

> **Document ID:** `114-debug-instance-prompt-running-detection/03-root-cause-analysis.md`  
> **Status:** APPROVED & APPLIED  
> **Target Subsystem:** Backend Rust (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/logger.rs`, `src-tauri/src/commands/instance.rs`)  
> **Reference Issue:** Task 114 — Debug Instance Prompt Running Detection and Multi-Instance Isolation  

---

## 1. Problem Description & Physical Failure Mode

In multi-profile Antigravity IDE environments (e.g., Sequence 1 Default Profile and Sequence 2 Cloned Profile `default-copy-8159`), the Antigravity Manager (AGM) user interface exhibited critical false-positive running indicators and cross-instance state pollution:

1. **Default Profile False Running State**:
   - The user launched the Default Profile to work exclusively on `Antigravity-Manager`.
   - `Antigravity-Manager` was actively executing prompts (`not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`).
   - Inactive/dormant projects in the default profile (`SpecBuilder` and `coding-guidelines`) had zero in-flight turns and had finished hours earlier.
   - **Defect**: AGM UI falsely rendered pulsing cyan `[RUNNING]` badges next to `SpecBuilder` and `coding-guidelines`.

2. **Instance 8159 False Running State**:
   - The user launched secondary instance `default-copy-8159` to work exclusively on `coding-guidelines`.
   - `coding-guidelines` was actively executing prompts (`not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`).
   - Cloned projects in 8159 (`Antigravity-Manager` and `SpecBuilder`) were dormant and not executing anything in this profile.
   - **Defect**: AGM UI falsely rendered pulsing cyan `[RUNNING]` badges next to `Antigravity-Manager` and `SpecBuilder`.

3. **Systemic Operational Impact**:
   - Automated profile rotation and idle watchers (e.g. `check_idle_projects_sensor`) were misled by false running states, failing to detect when workspaces were truly idle or prematurely firing fail-safes.
   - Fast-forward auto-resume erroneously resumed prompts from foreign instances onto dormant projects.
   - Users lost visibility into true machine activity across parallel developer sandboxes.

---

## 2. Deep Root Cause Analysis (The 5 Underlying Flaws)

An exhaustive audit of `src-tauri/src/modules/repo_db.rs` and related modules revealed five compounding architectural flaws that collectively caused this breakdown:

### Flaw 1: Timestamp Recency Heuristic Overriding Authoritative Idle Flags
- **Code Locations**:
  - `src-tauri/src/modules/repo_db.rs` -> `is_any_prompt_actively_running`
  - `src-tauri/src/modules/repo_db.rs` -> `is_prompt_running_for_project`
  - `src-tauri/src/modules/repo_db.rs` -> `compute_project_conversation_tree`
- **Flawed Code**:
  ```rust
  // Check recency within 600s
  if let Ok(mut stmt) = conn.prepare(
      "SELECT last_modified_time FROM conversation_summaries ORDER BY last_modified_time DESC LIMIT 10",
  ) {
      let now = Utc::now();
      let rows = stmt.query_map([], |row| row.get::<_, String>(0));
      if let Ok(rows) = rows {
          for time_str in rows.flatten() {
              if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&time_str) {
                  let age = (now - parsed.with_timezone(&Utc)).num_seconds();
                  if age >= 0 && age < 600 {
                      return true; // FLAW: Blind 10-minute recency override!
                  }
              }
          }
      }
  }
  ```
- **Why It Is Catastrophically Flawed**:
  Antigravity IDE's runtime engine explicitly writes `not_fully_idle = 0` and status `"CASCADE_RUN_STATUS_IDLE"` (or `"COMPLETED"`, `"FAILED"`, `"CANCELLED"`) to `conversation_summaries.db` the exact millisecond a turn finishes. The heuristic `age < 600` unconditionally declared any conversation modified in the last 10 minutes as `RUNNING`, completely ignoring the IDE's explicit idle declaration. Projects that completed execution 9 minutes ago continued to illuminate as `RUNNING`.

### Flaw 2: Scanning `antigravity-cli` Mixing Terminal Tasks into GUI Profiles
- **Code Location**: `src-tauri/src/modules/repo_db.rs` -> `gemini_dirs_for_instance`
- **Flawed Code**:
  ```rust
  for sub in ["antigravity", "antigravity-cli", "antigravity-ide"] {
      let path = home.join(".gemini").join(sub);
      if path.exists() {
          dirs.push(path);
      }
  }
  ```
- **Why It Is Catastrophically Flawed**:
  `antigravity-cli` is the local terminal assistant (`ag` / `antigravity-cli`). It maintains its own conversation history and SQLite databases for CLI coding operations. By including `antigravity-cli` in GUI instance directory candidate lists, independent CLI coding runs contaminated the GUI profile state, causing GUI project cards to pulse `[RUNNING]` when only a background terminal script was active.

### Flaw 3: Cloned `workspaceStorage` Inheriting Dormant Workspaces as Active
- **Code Location**: `src-tauri/src/modules/repo_db.rs` -> `detect_running_projects`
- **Flawed Code**:
  ```rust
  let is_project_active = is_instance_active
      && is_prompt_running_for_project(&raw_path, instance_id);
  ```
  Coupled with:
  ```rust
  let running_int = if p.is_running { 1 } else { 0 };
  INSERT OR REPLACE INTO running_projects ... VALUES (..., running_int, ...)
  // Inactive projects were never explicitly sanitized or demoted if already in table
  ```
- **Why It Is Catastrophically Flawed**:
  When an instance is cloned or reopened, `User/workspaceStorage` contains folders for every repository ever opened in VSCode/Antigravity IDE. Because `is_prompt_running_for_project` suffered from recency bleed and cross-instance matching, `is_project_active` evaluated to `true` for historical workspaces. Furthermore, `running_projects` rows were not actively demoted to `is_running = 0` when execution ceased.

### Flaw 4: Cross-Instance Database Bleeding & Un-Scoped Auto-Resume Queries
- **Code Location**: `src-tauri/src/modules/repo_db.rs` -> `auto_resume_recent_prompts`
- **Flawed Code**:
  ```rust
  let mut prompt_stmt = conn.prepare(
      "SELECT id, prompt_content, model, image_payload 
       FROM active_prompts 
       WHERE project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3
       ORDER BY created_at DESC LIMIT 1",
  )?;
  let maybe_prompt = prompt_stmt.query_row(
      rusqlite::params![&project.id, &project.repo_path, &proj_like],
      ...
  );
  ```
- **Why It Is Catastrophically Flawed**:
  The SQL query searched `active_prompts` strictly by project/path without restricting by `instance_id`. If `Antigravity-Manager` had an active prompt logged in instance `default`, querying auto-resume from instance `8159` matched the default instance's prompt and dispatched it into instance `8159`! Active prompt state bled bidirectionally across instance boundaries.

### Flaw 5: Unpartitioned Global Cache Keys
- **Code Location**: `src-tauri/src/modules/repo_db.rs` -> `get_project_conversation_tree_cached`
- **Flawed Code**:
  ```rust
  let cached = conn.query_row(
      "SELECT tree_json, updated_at, ttl_seconds FROM prompt_tree_cache WHERE cache_key = ?1",
      params![&cache_key],
      ...
  );
  ```
- **Why It Is Catastrophically Flawed**:
  The cache query did not filter by `instance_id = ?2`. If cache key normalization or un-scoped callers queried the tree, cached payloads generated for `default` were directly served to queries for `8159`.

---

## 3. Architecture & Invariant Fixes

```
+---------------------------------------------------------------------------------------------------+
|                                   Multi-Instance Isolation Engine                                 |
|                                                                                                   |
|  [Instance: default]                                            [Instance: 8159]                  |
|  dirs: ~/.gemini/antigravity, antigravity-ide                   dirs: instances/8159/home/...     |
|          |                                                               |                        |
|          v                                                               v                        |
|  gemini_dirs_tagged(Some("default"))                            gemini_dirs_tagged(Some("8159"))  |
|  [("default", ~/.gemini/antigravity)]                           [("8159", .../home/.gemini/...)]  |
|          \                                                               /                        |
|           +------------------------------+------------------------------+                         |
|                                          |                                                        |
|                                          v                                                        |
|             convs_by_inst_and_path: HashMap<(instance_id, repo_path), Vec<Conv>>                  |
|                                          |                                                        |
|             +----------------------------+----------------------------+                           |
|             |                                                         |                           |
|             v                                                         v                           |
|  Key: ("default", "Antigravity-Manager")                Key: ("8159", "coding-guidelines")        |
|  not_fully_idle=1, RUNNING -> is_running=true           not_fully_idle=1, RUNNING -> is_running=true
|                                                                                                   |
|  Key: ("default", "coding-guidelines")                  Key: ("8159", "Antigravity-Manager")      |
|  not_fully_idle=0, IDLE -> is_running=false             not_fully_idle=0, IDLE -> is_running=false|
|                                                                                                   |
|  Key: ("default", "SpecBuilder")                        Key: ("8159", "SpecBuilder")              |
|  not_fully_idle=0, IDLE -> is_running=false             not_fully_idle=0, IDLE -> is_running=false|
|                                          |                                                        |
|                                          v                                                        |
|                 Affirmative Liveness Gate: is_inst_alive && has_active_conv                       |
|                 (Zero recency overrides; explicit idle supremacy)                                 |
|                                          |                                                        |
|                                          v                                                        |
|                  Structured Audit Log: [InstancePromptAudit] via log_instance_prompt_audit         |
+---------------------------------------------------------------------------------------------------+
```

### Invariant 1: Strict Per-Instance Candidate Resolution (`gemini_dirs_tagged`)
- `gemini_dirs_for_instance(instance_id)` targets strictly `.gemini/antigravity` and `.gemini/antigravity-ide`. `antigravity-cli` is completely excluded.
- `gemini_dirs_tagged(instance_id)` returns pairs `(owning_inst_id, path)`. No file or database is read without an explicit, verifiable instance ownership tag.

### Invariant 2: Compound Key Grouping (`convs_by_inst_and_path`)
- All discovered conversations and active prompts are partitioned under compound key `(instance_id, normalized_repo_path)`.
- When assembling tree nodes for project `P` on instance `I`, only conversations under `(I, P.path)` can be attached. Cross-instance bleeding is mathematically prevented at the map key level.

### Invariant 3: Strict Idle Supremacy Rule
- If `not_fully_idle == 0` OR status contains `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, the turn is strictly **IDLE** (`is_conv_running = false`).
- Timestamp recency (`age < 600`) is permanently eliminated. An idle turn is never flipped to running based on elapsed time.

### Invariant 4: Decoupled Workspace Storage & Affirmative Execution
- A project discovered in `workspaceStorage` is marked running ONLY if:
  1. The underlying operating system process for that instance is alive (`is_inst_alive == true`), AND
  2. There is an active conversation in `conversation_summaries.db` (`has_active_conv == true`), OR
  3. `is_prompt_running_for_project` confirms affirmative in-flight execution for that exact `(instance_id, repo_path)`.
- Inactive projects are actively persisted with `is_running = 0` in `running_projects`.

### Invariant 5: Partitioned Cache & Scoped Auto-Resume
- Cache keys follow `tree:{instance_id}:{max_words}:{only_running}`. Both `cache_key` and `instance_id` are bound in SQLite queries.
- `auto_resume_recent_prompts` enforces strict instance scoping in SQL:
  ```sql
  WHERE (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
    AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = '__default__' OR instance_id = 'default' OR instance_id IS NULL OR instance_id = '')))
  ```

---

## 4. Verification & Prevention Patterns for Future AI Models

Future AI models and developers maintaining this codebase MUST uphold these non-negotiable architectural rules:

1. **NEVER use Timestamp Recency to Infer In-Flight Execution**:
   - Time elapsed since last modification does NOT signify activity. A developer reading a finished turn 2 minutes ago modified the file, but no prompt is executing. Always rely on explicit engine flags (`not_fully_idle != 0` and status `RUNNING`).

2. **NEVER Scan CLI Assistant Directories for GUI Profiles**:
   - `antigravity-cli` belongs exclusively to terminal workflows. Never mix `~/.gemini/antigravity-cli` into GUI instance candidate scans.

3. **NEVER Key Cross-Instance State by File Path Alone**:
   - Multiple instances can open the same repository. Keying maps or caches solely by `repo_path` guarantees state collision. Always qualify keys as `(instance_id, repo_path)`.

4. **NEVER Query `active_prompts` Without `instance_id`**:
   - Prompts belong strictly to the profile that created them. Every `SELECT`, `UPDATE`, or resume query against `active_prompts` must filter by `instance_id`.

5. **Mandatory Structured Forensic Logging**:
   - Every project liveness check must invoke `crate::modules::logger::log_instance_prompt_audit`:
     ```text
     [InstancePromptAudit] instance='8159' project='coding-guidelines' path='d:/work/coding-guidelines' is_instance_active=true is_running=true active_tasks=1 rationale='Active conversation summary with not_fully_idle=1'
     ```
   - This ensures immediate operational observability and eliminates guesswork during future debugging.

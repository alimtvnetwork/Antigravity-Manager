# Component Specification: Instance Prompt Status Detection & Audit Logging

## 1. Modified Backend Components (`src-tauri/src/modules/repo_db.rs`)

### 1.1 `gemini_dirs_for_instance`
- **Location**: `src-tauri/src/modules/repo_db.rs`
- **Signature**: `pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf>`
- **Changes**:
  - Enumerate subdirectories: `["antigravity", "antigravity-ide", "antigravity-cli"]`.
  - For `default` (or empty string/`__default__`), locate in user home `.gemini/`.
  - For named secondary instances, locate in `<instance_home>/.gemini/`.

### 1.2 `compute_project_conversation_tree`
- **Location**: `src-tauri/src/modules/repo_db.rs`
- **Signature**: `fn compute_project_conversation_tree(target_instance: Option<&str>, max_words: usize, only_running: bool) -> Vec<AgmProjectTreeNode>`
- **Changes**:
  - Namespace `seen_tree_cids` by `(String, String)`: `(owning_inst_id, cid)`.
  - In conversation row processing:
    - Parse `last_time_str` into UNIX timestamp using RFC3339, `%Y-%m-%d %H:%M:%S`, or `%Y-%m-%dT%H:%M:%S`.
    - Check turn freshness: `is_recent = timestamp >= now - 900`.
    - If `!is_recent`, unconditionally set `is_conv_running = false`.
    - Enforce idle supremacy: if `not_fully_idle == 0` or status contains `IDLE`, `COMPLETED`, `FAILED`, `CANCELLED`, set `is_conv_running = false`.
  - In project running decision:
    - `has_active_conv`: checks if any conversation assigned to this `(proj.instance_id, norm_path)` has `is_running == true`.
    - `has_active_prompt`: checks `is_prompt_running_for_project(&proj.repo_path, &proj.instance_id)`.
    - `proj_is_running = is_inst_alive && (has_active_conv || has_active_prompt)`.

### 1.3 `get_live_project_execution_info`
- **Location**: `src-tauri/src/modules/repo_db.rs`
- **Signature**: `pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo>`
- **Changes**:
  - Key `live_map` by `(String, String)`: `(instance_id, clean_path)`.
  - When matching projects from `running_projects`, look up `live_map.get(&(p.instance_id.to_lowercase(), clean_path))`.

### 1.4 `is_prompt_running_for_project`
- **Location**: `src-tauri/src/modules/repo_db.rs`
- **Signature**: `pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool`
- **Changes**:
  - Retain Gate 0: Host Process PID Liveness.
  - Retain Gate 1: Memory Prompts Map (TTL 300s).
  - Retain Gate 2: Active AGY Workers Map (PID verification).
  - Retain Gate 3: SQLite active_prompts (TTL 300s).
  - Retain Gate 4: Conversation Summaries (TTL 900s + Idle Supremacy), ensuring `gemini_dirs_for_instance` includes `antigravity-cli`.

## 2. Structured Audit Logging (`src-tauri/src/modules/logger.rs`)

### 2.1 `log_instance_prompt_audit`
- Format:
  `[InstancePromptAudit] instance='{}' (name='{}') project='{}' path='{}' source='{}' pid={:?} gate='{}' is_running={} rationale='{}'`
- Logged at every evaluation checkpoint and terminal determination.

## 3. End-to-End Tests (`src-tauri/tests/per_instance_prompt_liveness_test.rs`)

- Test exact multi-instance matrix:
  1. Default instance with active `Antigravity-Manager` -> default reports `Antigravity-Manager` running; does not report `SpecBuilder` or `coding-guidelines` running.
  2. 8159 instance with active `coding-guidelines` -> 8159 reports `coding-guidelines` running; does not report `SpecBuilder` or `Antigravity-Manager` running.
  3. Stale turn with `RUNNING` status (> 900s) -> reports `is_running = false`.
  4. Dead host process -> reports `is_running = false`.

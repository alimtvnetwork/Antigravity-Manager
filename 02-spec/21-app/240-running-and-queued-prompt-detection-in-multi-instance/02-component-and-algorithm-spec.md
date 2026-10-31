# Component and Algorithm Spec: Running and Queued Prompt Detection

## 1. Backend Components

### 1.1 `src-tauri/src/modules/repo_db/tree_collect.rs`
- **Liveness Decision Logic**:
  Refactor lines 145-168:
  ```rust
  let has_activity_marker = (cascade_status.is_running() || not_fully_idle > 0) && is_recent;
  let is_any_antigravity_alive = crate::modules::process::is_antigravity_running(None);
  let is_active_fallback = (is_owning_inst_alive || is_any_antigravity_alive) && has_activity_marker;

  let is_conv_running = if has_live_supremacy {
      true
  } else if is_active_fallback {
      // Affirmative running status from database and active IDE process trumps intermediate transcript idle markers
      true
  } else if is_terminal {
      false
  } else {
      false
  };
  ```

### 1.2 `src-tauri/src/modules/repo_db/tree_phases.rs`
- **Stale Prompt Transition**:
  Update `transition_stale_inflight_prompts(now: i64)`:
  - Do not purge `dispatched` or `queued` prompts after 120 seconds.
  - Set the timeout for `dispatched` prompts to at least 3600 seconds (1 hour).
  - Exempt prompts whose associated conversation ID has active running status.

### 1.3 `src-tauri/src/modules/instance/cdp_onboarding.rs`
- **CDP Queued Prompt Harvester**:
  Implement `extract_queued_composer_messages_for_instance(instance_id: &str) -> Vec<String>`:
  - Connects to the instance's active DevTools CDP WebSocket without origin restrictions (`Host: 127.0.0.1:{port}`).
  - Evaluates JS to locate the container holding "Queued Messages" text.
  - Extracts text contents of all queued prompt items (`.whitespace-pre-wrap` or `.line-clamp-2`).
  - Returns `Vec<String>` of queued prompt texts in FIFO sequence.

### 1.4 `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- **Queued Prompts Integration**:
  - In `collect_tree_conversation_items` and `tree_node_conv.rs`, harvest live queued messages from the IDE via CDP if the instance is running.
  - For each harvested prompt that is not already in the transcript turns:
    - Append as an `AgmPromptTurnNode` with `status: "QUEUED"`, `is_queued: true`, `is_running: false`.
  - Mark `target_conv.is_queued = true` and `target_conv.status = if target_conv.is_running { "RUNNING" } else { "QUEUED" }`.

### 1.5 `src-tauri/src/commands/instance.rs`
- Register `invalidate_prompt_tree_cache`:
  ```rust
  #[tauri::command]
  pub async fn invalidate_prompt_tree_cache(instance_id: Option<String>) -> Result<(), String> {
      reset_prompt_tree_cache(instance_id).await
  }
  ```

## 2. Frontend Components

### 2.1 `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- Enhance `isInstanceCompatible`:
  Allow instance ID matching across standard prefixes, numeric IDs, and trimmed aliases (e.g. `v3`, `v3-1180`, `#3 v3`).
- Prevent destructive wipe of optimistic queued turns on background SWR refreshes when the conversation is active.

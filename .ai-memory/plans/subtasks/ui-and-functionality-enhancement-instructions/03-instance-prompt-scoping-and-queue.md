# Subtask 03: Instance Prompt Scoping & Enqueued Prompts Pipeline

## Scope
- Files: `src-tauri/src/modules/repo_db/tree_phases.rs`, `src-tauri/src/modules/repo_db/tree_node_conv.rs`, `src-tauri/src/modules/repo_db/transcript.rs`, `src/pages/instances/instancePageUtils.ts`

## Implementation Steps
1. In `src-tauri/src/modules/repo_db/tree_phases.rs`:
   - In `transition_stale_inflight_prompts`, expand SQL query to transition stale `'dispatched'` active prompts older than 120 seconds to `'completed'`.
2. In `src-tauri/src/modules/repo_db/tree_node_conv.rs`:
   - In `build_active_prompt_conv_nodes`, add maximum TTL (10 minutes) freshness guard before setting `is_running = true`.
   - Forward `proj.instance_seq_num` to conversation nodes.
3. In `src-tauri/src/modules/repo_db/transcript.rs`:
   - Preserve `is_queued: true` for pending turns with step index beyond last completed model turn.
4. In `src/pages/instances/instancePageUtils.ts`:
   - Enforce strict instance ID scoping before alias fallback.

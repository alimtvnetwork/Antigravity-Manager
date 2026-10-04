# Subtask 003: Backend Liveness Scoping & Recency Gates (white-presentation-v1 Fix)

## Owner: Worker 02 (Backend Specialist)
## Target Files: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`

### Objective
1. **Root Cause Remediation for False Positive Liveness (`white-presentation-v1`)**:
   - In `src-tauri/src/modules/repo_db.rs`:
     - In `is_prompt_running_for_project` (lines ~1610-1850):
       - Gate 1 (Memory prompts): Check that `p.status == "running"` AND `now - p.updated_at <= 120` seconds.
       - Gate 2 (Active workers map): Only match if OS process PID is genuinely alive via sysinfo.
       - Gate 4 (Conversation summaries live turn):
         - Check that `not_fully_idle > 0` AND the conversation was updated recently (`now - summary_updated_at <= 120`). If the timestamp is stale (> 2 minutes ago), it MUST NOT be considered running!
     - In `get_project_execution_status` (lines ~2480-2525):
       - Remove loose substring matching on `pfx`:
         ```rust
         // Do not match if pfx is too short (< 6 chars) or if prefix is stale
         if pfx.len() >= 6 && pfx_inst == &norm_inst && (p.id.contains(pfx) || p.repo_path.contains(pfx)) {
             // Verify recency
             if now - last_time <= 120 {
                 is_running = true;
             }
         }
         ```
     - In `compute_project_conversation_tree` (lines ~4130-4155):
       - If `!is_inst_alive`, `proj_is_running` is strictly `false`.
       - If `is_inst_alive`, check if any conversation in `conv_nodes` has `effective_is_run == true` AND `now - last_modified <= 120`.
       - Avoid falling back to un-scoped historical records that keep `white-presentation-v1` permanently running under the Default instance.

# Subtask 003: Backend Liveness Scoping & Ghost Conversation Purge

## Owner: Worker 02 (Backend Specialist)
## Target Files: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`

### Requirements
1. **Strict 120s Recency & Process Liveness Gates**:
   - In `is_prompt_running_for_project`:
     - Gate 1: `(now - p.updated_at) <= 120`.
     - Gate 3: `now - 120` cutoff for SQLite active prompts.
     - Gate 4: `(now - conv_time) <= 120` for conversation summary turns.
   - In `get_project_execution_status`:
     - Discard prefixes with `len < 6`.
     - Verify `(now - last_time) <= 120`.
   - In `compute_project_conversation_tree`:
     - If `!is_inst_alive`, `proj_is_running` is strictly `false`.
     - Verify active conversation turn is within 120s before declaring `proj_is_running`.
2. **Filter & Merge Empty "Untitled Conversation" (0 Words) Nodes**:
   - In `compute_project_conversation_tree`, filter out conversation summaries and nodes where title starts with "untitled" (or empty) AND prompt content is empty or 0 words.
3. **Instance Metadata Parity**:
   - Ensure `AgmConversationNode` and `AgmProjectTreeNode` populate `instance_seq_num`, `instance_name`, `instance_exe_name`, and `prompt_tail_snippet`.

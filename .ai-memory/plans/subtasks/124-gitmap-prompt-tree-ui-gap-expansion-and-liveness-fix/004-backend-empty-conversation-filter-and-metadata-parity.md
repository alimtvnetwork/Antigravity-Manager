# Subtask 004: Backend Empty Conversation Filtering & Metadata Parity

## Owner: Worker 02 (Backend Specialist)
## Target Files: `src-tauri/src/modules/repo_db.rs`

### Objective
1. **Filter Out Empty "Untitled Conversation" (0 Words) Nodes**:
   - In `compute_project_conversation_tree` (lines 4007-4050):
     - Filter out conversation entries where `title.to_lowercase().starts_with("untitled")` (or empty) AND `raw_prompt.trim().is_empty()` (or word count == 0).
     - These ghost entries pollute the tree and confuse users. Only include meaningful conversations with prompt content.
2. **Add Instance Trio and Tail Snippet to Node Structures**:
   - In `AgmConversationNode`:
     - Add `instance_seq_num: Option<u32>`, `instance_name: String`, `instance_exe_name: String`, `prompt_tail_snippet: String`.
   - In `AgmProjectTreeNode`:
     - Add `instance_exe_name: String`.
   - Populate `instance_exe_name` by resolving:
     - If default instance: `'Antigravity.exe'` (Windows) or `'Antigravity'` (macOS/Linux) or path basename.
     - If custom instance: resolve from `inst.executable_path` (extract basename) or fallback to `format!("Antigravity-{}.exe", inst.id)`.
   - Compute `prompt_tail_snippet`:
     - Extract the last 10–15 words of the prompt preview and store in `prompt_tail_snippet`.

# Subtask 02: Running Projects Detection Root-Cause Fix

- **Objective**: Harden `src-tauri/src/modules/repo_db.rs` against false positive and false negative running project indicators.
- **Details**:
  - Remove `instance_id IS NULL OR instance_id = ''` from Default instance queries.
  - Require both process liveness and turn recency (<= 120s).
  - Filter out 0-word untitled conversations.
  - Flush stale `prompt_tree_cache` entries.

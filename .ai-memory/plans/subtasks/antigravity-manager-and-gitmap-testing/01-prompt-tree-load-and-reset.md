# Subtask 01 — Prompt tree load and reset

**Status:** Code written. Not click-tested.

1. Open Instances, then Prompts and History. The first scan may still take time. A second open within 120 seconds must use `prompt_tree_cache` and return without a full rescan.
2. If the scan panics, the window stays open and the error text includes `get_project_conversation_tree panicked`.
3. Reset on the tree header calls `reset_prompt_tree_cache` and reloads. Running and queued prompts remain.
4. `agm prompts reset-tree` prints `prompt tree cache cleared`.

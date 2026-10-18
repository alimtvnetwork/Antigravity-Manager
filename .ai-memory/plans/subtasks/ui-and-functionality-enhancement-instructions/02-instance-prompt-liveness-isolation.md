# Subtask 02: Instance Running Prompt Isolation & Multi-Instance Liveness Fix

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-02`  
**Status:** `READY`

### Objective
Ensure prompts running in Instance #2 never bleed into or falsely trigger running status on Instance #1 (Default):
1. In `src-tauri/src/modules/repo_db/gemini_dirs.rs`, de-couple `%USERPROFILE%/.gemini/antigravity-cli` from the default GUI IDE directory list. Tag CLI sessions with `"__cli__"`.
2. In `src-tauri/src/modules/repo_db/tree_node_conv.rs`, populate `instance_seq_num` using the instance registry so sequence numbers match frontend expectations.
3. In `src-tauri/src/modules/instance/observe.rs`, do not default empty instance specifiers to `default`.
4. In `src/pages/instances/instancePageUtils.ts` and `useInstancePage.ts`, strictly enforce 1:1 instance-to-node scoping.

### Target Files
- `src-tauri/src/modules/repo_db/gemini_dirs.rs`
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- `src-tauri/src/modules/repo_db/liveness.rs`
- `src-tauri/src/modules/instance/observe.rs`
- `src/pages/instances/instancePageUtils.ts`
- `src/pages/instances/useInstancePage.ts`

# Subtask 02: Instance Prompt Isolation & Liveness Scoping

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-02`  
**Status:** `READY`

## 1. Objectives & Scope
- Fix bug where running prompt is shown on both Instance #1 and Instance #2.
- In `src-tauri/src/modules/repo_db/gemini_dirs.rs`: do not conflate `antigravity-cli` with instance `default`. Isolate CLI sessions.
- In `src-tauri/src/modules/repo_db/prompts_crud.rs`: inspect `project_id` and extract instance discriminator (e.g. `__default-copy-5597`) so `active_prompts.instance_id` is set to the correct instance ID rather than defaulting to `default`.
- In `src-tauri/src/modules/repo_db/liveness.rs`: ensure Gate 3 strictly matches target instance ID.
- In `src-tauri/src/modules/repo_db/tree_phases.rs`: filter active prompts strictly by instance ID in `discover_fallback_projects`.

## 2. Disjoint File Ownership
- `Owned Files:` `src-tauri/src/modules/repo_db/gemini_dirs.rs`, `src-tauri/src/modules/repo_db/prompts_crud.rs`, `src-tauri/src/modules/repo_db/liveness.rs`, `src-tauri/src/modules/repo_db/tree_phases.rs`

## 3. Verification Evidence Contract
- Querying prompt liveness for Instance #1 returns false when the prompt is running only in Instance #2.

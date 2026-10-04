# Subtask 01: Core Detection Logic Refactor & Instance Isolation

## Objective
Refactor `src-tauri/src/modules/repo_db.rs` to eliminate cross-instance running prompt bleed, enforce a strict 15-minute TTL on conversation turns in `compute_project_conversation_tree`, add `antigravity-cli` to candidate directories in `gemini_dirs_for_instance`, and strictly scope `live_map` and conversation resolution by `(instance_id, repo_path)`.

## Scope of Work
1. **TTL Enforcement in `compute_project_conversation_tree`**:
   - Parse `last_time_str` against `now - 900`.
   - If turn timestamp is older than 900 seconds, force `is_conv_running = false`.
   - Strictly prioritize idle statuses (`IDLE`, `COMPLETED`, `FAILED`, `CANCELLED`) or `not_fully_idle == 0`.
2. **Include `antigravity-cli` in `gemini_dirs_for_instance`**:
   - Add `"antigravity-cli"` alongside `["antigravity", "antigravity-ide"]` so CLI runs are discovered for default and named instances.
3. **Strict Instance Scoping in `get_live_project_execution_info`**:
   - Key `live_map` by `(instance_id, clean_path)` rather than path alone.
   - Disallow conversations from instance A from marking projects running on instance B.
4. **Namespace Conversation Deduplication**:
   - Change `seen_tree_cids` to track `(owning_inst_id, cid)` to prevent accidental cross-instance suppression.
5. **Project-Level Running Decision Hardening**:
   - In `compute_project_conversation_tree`, ensure `proj_is_running` requires `is_inst_alive && (has_active_conv || has_active_prompt)`.
   - Ensure `has_active_conv` only checks conversations belonging strictly to `(proj.instance_id, proj.repo_path)`.
   - Invalidate stale `prompt_tree_cache` entries when recomputing to prevent caching stale running state.

## Acceptance Criteria
- [ ] Stale turns (> 900s) are never marked `is_running = true`.
- [ ] Default instance reports ONLY `Antigravity-Manager` running; `SpecBuilder` and `coding-guidelines` report idle.
- [ ] 8159 instance reports ONLY `coding-guidelines` running; `Antigravity-Manager` and `SpecBuilder` report idle.
- [ ] Zero compiler errors or clippy warnings.

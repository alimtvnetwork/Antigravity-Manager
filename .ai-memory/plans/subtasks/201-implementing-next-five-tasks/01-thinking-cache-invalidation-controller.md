# Subtask 01: Thinking Cache Invalidation Control
Status: COMPLETED
Sequence: 201
Owner: Worker-1
Target Module: `src-tauri/src/modules/release_gate/thinking_cache_invalidation_controller.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_thinking_cache_invalidation_controller.rs`
Objective: Enforce AGENTS.md thinking cache invalidation control (发版清理建议): Routine releases (no prompt): Keep `SUGGESTION_DELETE_THINKING_STORE = false`. Architecture / schema refactors: Set `SUGGESTION_DELETE_THINKING_STORE = true`, Set `SUGGESTION_TARGET_VERSION = '<version>'`. On upgrade, users with existing cache get a one-time prompt; action state persists in `gui_config.json`.

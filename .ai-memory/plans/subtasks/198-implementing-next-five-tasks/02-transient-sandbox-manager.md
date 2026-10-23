# Subtask 02: Transient Sandbox Manager & Zero-Orphan Cleanup
Status: COMPLETED
Sequence: 198
Owner: Subagent-2
Target Module: `src-tauri/src/modules/instance/transient_sandbox_manager.rs`
Test Suite: `src-tauri/src/modules/instance/tests_transient_sandbox_manager.rs`
Objective: Implement lifecycle management and complete zero-orphan cleanup for transient test sandboxes (`test-create` / `test-clean`), preserving `workspaceStorage`.

Evidence:
- Module `src-tauri/src/modules/instance/transient_sandbox_manager.rs` created with `create_transient_sandbox`, `clean_transient_sandbox`, `sweep_stale_sandboxes`, and `evaluate_caller_protection`.
- Test suite `src-tauri/src/modules/instance/tests_transient_sandbox_manager.rs` created covering caller IDE preservation, force override, sandbox lifecycle, workspaceStorage preservation, and stale sandbox sweeping.
- Registered and re-exported in `src-tauri/src/modules/instance.rs`.

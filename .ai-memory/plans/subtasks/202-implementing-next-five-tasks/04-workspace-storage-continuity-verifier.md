# Subtask 04: Headless & CLI Profile Workspace Storage Continuity Verifier
Status: COMPLETED
Sequence: 202
Owner: Worker-2
Target Module: `src-tauri/src/modules/instance/workspace_storage_continuity_verifier.rs`
Test Suite: `src-tauri/src/modules/instance/tests_workspace_storage_continuity_verifier.rs`
Objective: Enforce AGENTS.md headless & CLI parity: Ensure GUI configurations and lifecycle operations maintain functional parity across headless servers, CLI environments, and cross-platform environments. Any profile/instance duplication or settings synchronization must pass through identical underlying service routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`) and allowing deep-merge settings synchronization from both GUI and CLI.
Lines: 147 lines implementation + 144 lines unit tests.

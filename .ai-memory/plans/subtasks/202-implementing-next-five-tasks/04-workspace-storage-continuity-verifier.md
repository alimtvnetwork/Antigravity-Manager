# Subtask 04: Headless & CLI Profile Workspace Storage Continuity Verifier
Status: IN PROGRESS
Sequence: 202
Owner: Worker-2
Target Module: `src-tauri/src/modules/instance/workspace_storage_continuity_verifier.rs`
Test Suite: `src-tauri/src/modules/instance/tests_workspace_storage_continuity_verifier.rs`
Objective: Enforce AGENTS.md headless & CLI parity: Any profile/instance duplication or settings synchronization must pass through identical underlying service routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`) and allowing deep-merge settings synchronization from both GUI and CLI.

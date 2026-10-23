# Subtask 02: Caller IDE Self-Preservation Exit Code Enforcer
Status: IN PROGRESS
Sequence: 203
Owner: Worker-1
Target Module: `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs`
Test Suite: `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs`
Objective: Enforce AGENTS.md caller IDE self-preservation invariant: Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided, preventing developer workflow disruption.

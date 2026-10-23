# Subtask 04: Caller IDE Exit Code 2 Enforcer (`Imp-693`)
Sequence: 213
Step: 279
Status: IN_PROGRESS
Module: `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs`
Test Suite: `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs`

## Description
Enforces AGENTS.md rule: "Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided." Evaluates instance lifecycle actions, checks whether target matches caller instance ID, validates presence of `--force` / `force: bool`, returns structured interception reports, and formats user-friendly error diagnostics.

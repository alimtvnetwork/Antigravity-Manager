# Subtask 04: Caller IDE Exit Code 2 Enforcer (`Imp-693`)
Sequence: 213
Step: 279
Status: COMPLETED
Module: `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs`
Test Suite: `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs`

## Description
Enforces AGENTS.md rule: "Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided." Evaluates instance lifecycle actions, checks whether target matches caller instance ID, validates presence of `--force` / `force: bool`, returns structured interception reports, and formats user-friendly error diagnostics.

## Implemented Components
1. `CALLER_PROTECTION_EXIT_CODE: i32 = 2`: Defined standard non-zero exit code constant.
2. `InstanceCommandType` enum: Represents destructive actions (`Stop`, `Remove`, `Switch`, `Restart`) and non-destructive operations (`NonDestructive(String)`), implementing `is_destructive()` and `as_str()`.
3. `CallerInstanceContext` struct: Encapsulates caller session metadata (`caller_instance_id`, `caller_pid`, `caller_ide_name`) and target matching (`is_caller()`).
4. `CommandInterceptionResult` struct: Encapsulates `is_intercepted`, `exit_code`, `target_instance_id`, `is_caller_match`, `was_force_provided`, `reason`, `remediation_hint`, with `format_terminal_error()` and `Display` implementations.
5. `evaluate_instance_command`: Evaluates commands targeting instances:
   - When target matches caller instance ID and command is destructive and `!force`: returns `exit_code: 2`, `is_intercepted: true` with actionable remediation hint.
   - When `force == true` or `target != caller`: returns `exit_code: 0`, `is_intercepted: false`.
   - When command is non-destructive (e.g. status): returns `exit_code: 0`, `is_intercepted: false`.
6. Legacy compatibility preserved for `TargetInstanceCommand`, `CallerProtectionPolicy`, `CallerProtectionVerdict`, and `evaluate_instance_action`.
7. Comprehensive unit test suite in `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs`.

# Subtask 01: Spec & Backend Instance Process Teardown

## Status: Completed

## Parent Plan
[Plan 48: Instance Smart Rotation, Process Termination, Account Selection Algorithm & Minor Release v4.37.0](.ai-memory/plans/completed/48-instance-smart-rotation-account-switch.md)

## Goal
Implement thorough, reliable backend process teardown for instance profiles based on their specific filesystem location and `--user-data-dir` argument.

## Key Actions
1. Author architectural specification at `02-spec/20-instance-management/02-smart-rotation-account-switch-spec.md`.
2. Update `src-tauri/src/modules/instance.rs`:
   - Enhance `close_instance` to use `taskkill /F /T /PID` (tree-kill) on Windows so child processes, renderers, language servers, and terminal hosts do not survive.
   - Enhance `find_pids_for_data_dir` to ensure processes running custom executables from `instance_bin_dir` or with `--user-data-dir` matching `clean_target` are matched accurately.
3. Verify process termination returns cleanly when no processes are running.

## Constraints
- Total ban on explicit boolean comparisons against `true`.
- Zero absolute paths or `file:///` URIs.
- Preserve all existing tests and error handling.

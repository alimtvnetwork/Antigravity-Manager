# Subtask 03: Caller Instance Detection & Self-Preservation in CLI

Owner: Worker 02
Target Files:
- `src-tauri/src/modules/instance/close.rs`
- `src-tauri/src/bin/agm/instance_cmds.rs`
- `src-tauri/src/bin/agm/switch_check_cmds.rs`

## Requirements
1. In `src-tauri/src/modules/instance/close.rs`:
   - Enhance `detect_caller_instance_id()`:
     - Check parent/ancestor processes for executable matching `antigravity.exe` or `antigravity`.
     - Inspect command-line arguments of ancestor processes using `sysinfo` (`proc.cmd()`).
     - Look for `--user-data-dir` or `-user-data-dir`:
       - If `--user-data-dir <path>` matches an instance's `data_dir` in `registry.instances`, return that instance's ID.
       - If no `--user-data-dir` argument is passed on an ancestor `antigravity.exe`, return `"default"`.
     - Check ancestor PIDs against `inst.pid` in `registry.instances`.
     - Check environment variables (`ANTIGRAVITY_INSTANCE_ID`, `AGM_INSTANCE_ID`) if set.
2. In `refuse_if_caller_instance(target_id)`:
   - When caller is detected, prevent stopping, switching, or deleting that instance unless `--force` is specified.
   - Print clear explanation: `[BLOCKED] Refusing to stop or mutate instance '{self_id}' because this command is running from inside its IDE process.`
3. Wire `guard_caller_instance` across `cmd_instances_stop`, `cmd_instances_delete`, and CLI switch commands.

## Status: COMPLETED
- Hardened `detect_caller_instance_id()` with ancestor process tree inspection, `proc.cmd()` for `--user-data-dir`, and environment variable checks.
- Enforced caller self-preservation in `refuse_if_caller_instance` and instance CLI commands.

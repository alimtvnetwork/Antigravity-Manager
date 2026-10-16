# RCA 17: Instance list PID, stop, switch, and empty audit trail

Fixed in v4.209.0 (`52aaf4cb`). Spec: `02-spec/21-app/instance-management-and-ui-ux-synchronization/01-architecture-spec.md`.

## Symptom

The list badge showed `Running · 1632` with no label. The executable cell showed `...\Antigravity...` while the tooltip held the full filename. Stop Instance left the IDE running. Account switch did not close the live process before writing credentials. The instance audit dialog said "No Account Switches Recorded" for Default.

## Root cause

Three independent mismatches. None of them was a missing PID or an empty switch database.

1. **The number is the PID, and the label omitted that word.** `StatusCell` in `src/components/instances/table/InstanceRowCells.tsx` rendered `Running · ${inst.pid}`. `1632` is `inst.pid`, the OS process id stored on the instance. `formatShortPath` already returned the last path segment, then CSS `truncate` inside a column about 130px wide cut a long name such as `Antigravity-default-copy-5597.exe` in the middle.

2. **Stop and switch treated a filtered-out PID as a successful close.** `close_instance` in `src-tauri/src/modules/instance/close.rs` added the saved PID only when the command line or executable contained the data directory or the instance id. Electron does not put `--user-data-dir` on every process, so the PID shown in the list was dropped. An empty PID list called `mark_instance_stopped` and returned `Ok` without `taskkill`. The next refresh read the live OS process and the row still said Running. `switch_account_to_instance` called `close_instance_verified` and, on `Ok(false)`, logged a warning and wrote credentials while the old process was still alive.

   A second check made a real kill look like a failure. The wait loop and `is_pid_alive_os` used sysinfo `refresh_processes_specifics` on a single PID. On Windows that snapshot keeps a process that has already exited. The first stop of `test-e2e-alpha-9746` returned "stayed alive" for PIDs that `taskkill` then reported as not found.

3. **The audit query returned rows the dialog did not read.** Switches are inserted with `action_code = 3` and `instance_id` by `AuditTask::start`. `get_instance_switch_history` serialized `total`, `records`, and `items`. `InstanceAuditTrailModal` only read `total_switches` and `recent_switches`. Those fields were absent, so the dialog took the empty-state branch.

## How found

- List rendering: `StatusCell` and `formatShortPath` in `src/components/instances/table/`.
- Stop path: `close_instance` drops the saved PID, then returns `Ok` when `pids` is empty. `stop_instance` called that function. Switch continued after `close_instance_verified` returned `false`.
- Audit path: Rust `InstanceSwitchHistoryResponse` field names versus `recent_switches` in `src/types/audit.ts` and `InstanceAuditTrailModal.tsx`.
- Live check on a new instance `e2e-pid-sync-941`: start saved PID `4880` plus five children; stop after the exit-code liveness check killed all six and confirmed them dead. Default and Default Copy were left running. `agm instances current` printed `none` from this Cursor session because the parent chain is not an Antigravity instance.

## Fix

- Badge text is `PID <n>`. Path cells keep the filename ending (`formatPathEnding`).
- A saved Antigravity PID is included when its executable belongs to that instance. Stop uses `close_instance_verified` and returns an error if the PID remains. Switch returns an error instead of injecting credentials. Windows liveness uses `OpenProcess` and `GetExitCodeProcess` (`STILL_ACTIVE` is 259).
- The history response also emits `recent_switches` and `total_switches`.
- `agm instances current` walks parent PIDs and executables. Stop, switch, restart, and delete refuse that instance.

## Verification

- `cargo fmt -- --check` and `cargo check -j 1 --bin agm` passed.
- `npm run build` passed after the audit modal null check.
- Stop of `e2e-pid-sync-941` logged six PIDs, then `confirmed_dead=true` in 608ms. The test instance and the idle `test-e2e-alpha` and `test-e2e-beta` rows were deleted. Default stayed on PID `9656`. Default Copy stayed on PID `6376`.

## Prevention

- Do not mark an instance stopped when the saved PID was skipped. An empty kill list is success only when the OS shows no matching process.
- Do not treat a sysinfo single-PID refresh as proof a Windows process is alive. Use the process exit code.
- Do not rewrite instance credentials when verified close returns false.
- The audit dialog and `InstanceSwitchHistoryResponse` must share `recent_switches` and `total_switches`. Adding a Rust field without the name the modal reads recreates the empty state.

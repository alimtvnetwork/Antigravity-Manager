# Subtask 01: Instance Process PID Tracking & Selective Process Termination

Traceability ID: Task-01
Target Files:
- src-tauri/src/modules/instance.rs
- src-tauri/src/modules/process.rs
- src-tauri/src/commands/instance.rs

Action:
- Add `pid: Option<u32>` to `InstanceConfig` and persist to SQLite `instances.db`.
- When an instance is launched via `launch_instance_by_id`, capture `child.id()` and update the instance record with `pid = Some(child_pid)`.
- Replace blanket `taskkill /F /IM antigravity.exe` and `killall antigravity` in instance stop/switch routines with targeted process termination targeting only the recorded PID (`taskkill /PID <pid> /T /F` on Windows and `kill -9 <pid>` on Unix).
- Clean up instance PID upon process termination or detected death.

Acceptance Criteria:
- Spawned instances store their exact root PID in `instances.db`.
- Stopping an instance terminates only processes belonging to that PID tree without affecting sibling Antigravity instances.
- Sibling instances with different profiles remain running continuously during single-instance stop or switch operations.

Targeted Verification:
- python 03-ai-scripts/05-guideline-autofixer.py src-tauri/src/modules/instance.rs

Status: COMPLETED
- Added `pid: Option<u32>` to `InstanceConfig` and SQLite DB `instances.db` (`instance_processes` table).
- Isolated PID tracking in `launch_instance` using `child.id()`.
- Selective process termination in `close_instance` with `taskkill /F /T /PID <pid>`.
- Verified no collateral termination of sibling instances.

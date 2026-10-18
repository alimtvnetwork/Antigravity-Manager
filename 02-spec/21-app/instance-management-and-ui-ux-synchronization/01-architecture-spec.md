# Instance Management & UI/UX Synchronization — Architecture Specification

## 1. Scope & Objective
This specification establishes the architectural foundation for deterministic Antigravity IDE instance lifecycle management, process termination, pre-switch process eviction, caller IDE self-preservation, and full-stack instance audit logging in Antigravity-Manager.

## 2. Core Architectural Pillars

### 2.1 PID Semantics & List View Layout
- **Semantic Definition:** Numeric status values (e.g. `1632`) unambiguously represent the native Operating System **Process ID (`PID`)** of the active Antigravity IDE instance.
- **List View Single-Row Invariant:** In table/list views (`InstanceTable.tsx` / `InstanceRowCells.tsx`), the status indicator with `PID: <pid>`, the trailing executable path (`.../Antigravity.exe`), and the trailing user data directory (`.../antigravity-data`) MUST reside together on the exact same horizontal flex row, separated by subtle `·` glyphs, with copy action buttons grouped into a contiguous segmented pill capsule.

### 2.2 Deterministic Multi-Stage Termination Ladder
Process termination on Windows must not rely on a single call or assume instantaneous kernel cleanup. The system enforces a 5-stage progressive ladder:
1. **Lineage Identification:** Discover root process PID and all child/descendant PIDs via `sysinfo` and process parent maps.
2. **Graceful & Broad Tree Kill:** Windows invokes `taskkill /F /T /PID <root_pid>` with `CREATE_NO_WINDOW (0x08000000)`. Unix invokes `kill -15 <pid>`.
3. **Initial Verification Window:** Fast poll loop every 50ms (up to 1,500ms) checking `GetExitCodeProcess` for `STILL_ACTIVE (259)`.
4. **Force Escalation Fallback:** If any PIDs survive, escalate immediately:
   - Win32 direct kernel kill: `OpenProcess(PROCESS_TERMINATE, 0, pid)` + `TerminateProcess(h, 1)`.
   - PowerShell fallback: `Stop-Process -Id <pid> -Force`.
   - Unix fallback: `kill -9 <pid>`.
   - Second poll window: every 100ms (up to 2,500ms).
5. **Post-Kill Settle Delay & Sanitation:**
   - Enforce mandatory 200ms kernel settle delay (`std::thread::sleep(Duration::from_millis(200))`).
   - Unconditionally clear stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`, `Singleton*`).
   - Update SQLite `instance_processes` marking `is_active = 0` and `pid = NULL`.
   - Invalidate process caches and emit Tauri IPC refresh events.

### 2.3 Pre-Switch Process Eviction & Lockfile Hygiene
To eliminate credential write-back races and SQLite database locked errors on `state.vscdb`:
1. Verify whether target instance has an active PID.
2. If running, execute verified termination ladder before touching `state.vscdb`.
3. Clear all lockfile artifacts.
4. Inject new account credentials with a 3-attempt retry loop.
5. Relaunch instance preserving original workspace configurations.
6. Poll and synchronize new materialized PID into SQLite.

### 2.4 Caller IDE Process Self-Preservation
- Automated switchers, CLI tests, and remote daemons must NEVER terminate or alter the IDE process inside which the developer or AI agent is actively executing.
- **Ancestor Walk Detection:** Inspect current process lineage up to 24 generations. Match PIDs, executable names (`antigravity.exe`), `--user-data-dir` arguments, and cloned binary markers.
- **Immunity Guard (`refuse_if_caller_instance`):** When caller PID or instance ID matches target, immediately refuse the operation with exit code 2 and a structured JSON error (`REFUSE_CALLER_INSTANCE`), bypassable only with `--force`.

### 2.5 Split-DB Sharded Audit Trail
- All lifecycle events (`START`, `STOP`, `RESTART`, `KILL`, `CREATE`, `CLONE`, `DELETE`, `SWITCH`) are logged into `task_history_db` with `instance_id`.
- Action codes 1 to 14 map to structured `AuditAction` enum variants.
- Split-DB catalog (`task_index.db` and `history-*.db`) query functions expand aliases (`""`, `NULL`, `"default"`, `"antigravity"`, `"agy"`, `"ide"`) to ensure zero missing audit records.
- UI modal displays dual tabs: `Switch Cycles` and `All Operations`, with automatic fallback to operations when switch cycles are 0.

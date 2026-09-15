# Multi-Instance Antigravity & Profile Isolation Blueprint

> Feasibility analysis, technical mechanics, and implementation blueprint for running multiple concurrent Antigravity instances with distinct profiles.

---

## 1. Executive Summary & Feasibility

**Question:** *Can we run multiple instances of Antigravity with different profiles at the same time?*

**Verdict: YES, completely feasible.**

Antigravity is built on top of the VS Code / Electron desktop platform. While the default Antigravity-Manager configuration operates in single-instance mode (terminating previous processes before switching accounts), the underlying architecture natively supports simultaneous, isolated multi-instance execution.

---

## 2. Why the Default Setup Runs Single-Instance

In the existing codebase (`src-tauri/src/modules/process.rs` and `integration.rs`), three constraints enforce single-instance behavior:

1. **Shared SQLite File Locks:** Both Antigravity and Antigravity-Manager target the default `%APPDATA%\Antigravity\User\globalStorage\state.vscdb`. Writing to this file while an instance is running triggers database lock contention.
2. **Electron Single-Instance Lock:** Electron activates a cross-process named pipe/mutex (`antigravity-main`) on startup. Launching a second executable without custom profile arguments simply forwards window focus to the existing process.
3. **Global OS Keyring Target:** The OS credential manager entry (`gemini:antigravity`) provides only one global slot per OS user account.

---

## 3. The Multi-Instance Architecture Blueprint

To run multiple concurrent instances with isolated profiles and tokens, Antigravity must be launched with profile sandboxing:

```mermaid
graph TD
    subgraph Antigravity-Manager [Antigravity-Manager Orchestrator]
        AM_Core["Profile Supervisor"]
        Acc1["Account A (work@corp.com)"]
        Acc2["Account B (personal@gmail.com)"]
    end

    subgraph FileSystem [Isolated Storage Directory Tree]
        P1["%LOCALAPPDATA%/Antigravity-Profiles/profile-work/"]
        P2["%LOCALAPPDATA%/Antigravity-Profiles/profile-personal/"]
        DB1["P1/.../state.vscdb (Account A Token)"]
        DB2["P2/.../state.vscdb (Account B Token)"]
    end

    subgraph Runtime [Concurrent Antigravity Electron Processes]
        Inst1["Antigravity PID 10420<br/>--user-data-dir P1<br/>Token: Account A"]
        Inst2["Antigravity PID 18944<br/>--user-data-dir P2<br/>Token: Account B"]
    end

    AM_Core -->|Injects Token A| DB1
    AM_Core -->|Injects Token B| DB2
    AM_Core -->|Spawns with P1 args| Inst1
    AM_Core -->|Spawns with P2 args| Inst2
    Inst1 -.->|Reads state| DB1
    Inst2 -.->|Reads state| DB2
```

---

## 4. Step-by-Step Implementation Mechanics

### Step 1: Partitioned User Data Directories

For each registered account in Antigravity-Manager, provision an isolated data folder:

- **Windows:** `%LOCALAPPDATA%\Antigravity-Manager\profiles\<account_id>\`
- **macOS:** `~/Library/Application Support/Antigravity-Manager/profiles/<account_id>/`
- **Linux:** `~/.config/antigravity-manager/profiles/<account_id>/`

### Step 2: Direct SQLite Token Injection

Instead of modifying the shared host OS keyring, inject the account's OAuth credentials directly into the profile's dedicated database:

1. Target path: `<profile_dir>/User/globalStorage/state.vscdb`.
2. Ensure directory tree exists (`fs::create_dir_all`).
3. Call `inject_token()` (`src-tauri/src/modules/db.rs`) with the profile-specific `db_path`.
4. Result: Instance A boots immediately authenticated as Account A; Instance B boots as Account B.

### Step 3: CLI Spawning Arguments

Launch each Antigravity instance with isolated CLI flags via `src-tauri/src/modules/process.rs`:

```bash
antigravity.exe \
  --user-data-dir "C:\Users\<user>\AppData\Local\Antigravity-Manager\profiles\acc_01\data" \
  --extensions-dir "C:\Users\<user>\AppData\Local\Antigravity-Manager\profiles\acc_01\extensions" \
  --password-store="basic" \
  --new-window
```

| Flag | Purpose |
| :--- | :--- |
| `--user-data-dir <path>` | Diverts SQLite, localStorage, session cookies, and workspace state into an isolated folder. Bypasses the single-instance mutex. |
| `--extensions-dir <path>` | Allows sharing or isolating installed extensions per profile. |
| `--password-store="basic"` | Forces Chromium/Electron to isolate credentials in local profile storage rather than colliding on the host OS credential manager. |
| `--new-window` | Prevents the second process from attaching to an existing main process. |

---

## 5. Supervisor Logic Adjustments in Manager

To support this in Antigravity-Manager, update `src-tauri/src/modules/process.rs`:

```mermaid
flowchart TD
    Req["Request: Launch Profile (account_id)"] --> CheckRunning{"Is instance for account_id already running?"}
    
    CheckRunning -- "Yes" --> FocusWin["Find existing window for account_id and bring to foreground"]
    
    CheckRunning -- "No" --> PrepareData["1. Ensure profile directory exists<br/>2. Inject Token into profile state.vscdb<br/>3. Set window title in profile settings.json"]
    
    PrepareData --> LaunchProc["Spawn process::start_antigravity with args:<br/>--user-data-dir <profile_path>"]
    
    LaunchProc --> TrackPID["Record PID in ActiveInstances Map<br/>[account_id -> PID]"]
```

### Safety Guard
Modify `close_antigravity()`: when launching Profile B, do **not** kill all Antigravity processes. Only terminate the process matching the specific `account_id` if a restart is requested for that specific profile.

---

## 6. Verification Checklist

1. [ ] Launch Profile A (`user1@domain.com`) with `--user-data-dir <dirA>`.
2. [ ] Launch Profile B (`user2@domain.com`) with `--user-data-dir <dirB>`.
3. [ ] Verify both windows run simultaneously in Windows Task Manager with independent PIDs.
4. [ ] Send AI prompts in both windows; confirm quotas decrement from the respective distinct accounts.

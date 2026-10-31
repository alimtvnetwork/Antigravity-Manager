# Specification: Instance Lifecycle, PID Termination & Account Switch Synchronization

## 1. Executive Overview
This specification details the mechanics of process tracking, PID termination ladders, and account switching synchronization within Antigravity-Manager.

## 2. PID Confirmation & Display Contract
- **Confirmation:** The numeric indicator (e.g. `1632`) represents the live Operating System Process ID (PID) of the Antigravity instance (`Antigravity.exe`).
- **UI Display Invariant:**
  - In `InstanceRow.tsx` (list view), when `instance.is_running` is true and `instance.pid` is present, the status badge MUST explicitly state `PID: <pid>` (e.g. `PID: 1632`) alongside the green pulse icon.
  - Path alignment invariant: The executable path ending (e.g. `.../Antigravity.exe`) and data directory path ending (e.g. `.../data`) MUST be displayed together on the exact same line within a single flex row container, separated by a middle dot (`·`) or subtle divider.

## 3. Account Switching & Stop Instance Termination Ladder
- **Root Problem:** When switching accounts or stopping an instance, if an existing process is alive or PID is mismatched, credentials cannot be swapped cleanly without file lock conflicts.
- **Enforced Termination Flow:**
  1. Inspect `instance.pid` and verify if the process is alive.
  2. If running, execute tree termination (`taskkill /F /T /PID <pid>` on Windows, `kill -9` on Unix).
  3. Wait up to 1500ms for process handle closure.
  4. Perform atomic storage update / account binding swap in `storage.json` and SQLite.
  5. Clear volatile state (`state.vscdb` session markers if requested).
  6. Relaunch instance only if restart was explicitly requested.
- **Stop Instance Action Invariant:**
  - The "Stop Instance" button in GUI and CLI MUST terminate the process tree immediately and set `is_running = false` and `pid = None` in the database and active state.

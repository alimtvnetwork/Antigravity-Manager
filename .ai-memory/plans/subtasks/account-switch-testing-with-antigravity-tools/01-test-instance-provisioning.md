# Subtask 01: Test Instance Provisioning (COMPLETED)

## Status: COMPLETED

## Objective

Safely provision an isolated test instance sandbox named `test-switch-verify` using `agm instances test-create switch-verify --json` to verify sandboxing, clean profile directory allocation, keyring bypass configuration, and complete non-interference with the developer's working instance (`default`, PID 13632 immunity).

---

## Detailed Execution Steps & Concrete Evidence

### 1. Caller IDE Self-Preservation Verification (Pre-Flight Guard)
- **Ancestor Detection**: Confirmed that `detect_caller_instance_id()` correctly resolves the developer's active workspace session (`default`, PID 13632).
  ```bash
  .\src-tauri\target\debug\agm.exe instances whoami --json
  ```
  **Output**:
  ```json
  {
    "success": true,
    "detected": true,
    "id": "default",
    "instance_id": "default",
    "name": "Default",
    "pid": 13632,
    "data_dir": "C:\\Users\\Administrator\\AppData\\Roaming\\Antigravity",
    "bound_email": "cirajayu11@gmail.com"
  }
  ```
- **Refusal Test**: Verified that attempting to stop the caller instance without `--force` fails immediately:
  ```bash
  .\src-tauri\target\debug\agm.exe instances stop default
  ```
  - **Result**: Command refused and aborted with **exit code 2** ($LASTEXITCODE = 2):
    ```
    [ERROR] Refusing to stop, switch, restart, or delete instance 'default' because this command is running inside its IDE process. Target instance 'default' is your currently running IDE instance! Aborting to prevent terminating your own session. Pass --force to override.
    ```

### 2. Sandbox Provisioning (`agm instances test-create`)
- **Execution Command**:
  ```bash
  .\src-tauri\target\debug\agm.exe instances test-create switch-verify --json
  ```
- **Response Validation**: Parsed the returned JSON envelope:
  ```json
  {
    "success": true,
    "id": "test-switch-verify-1792432990-2990",
    "name": "test-switch-verify-1792432990",
    "data_dir": "C:\\Users\\Administrator\\.antigravity_tools\\instances\\test-switch-verify-1792432990-2990\\data",
    "config": {
      "id": "test-switch-verify-1792432990-2990",
      "name": "test-switch-verify-1792432990",
      "data_dir": "C:\\Users\\Administrator\\.antigravity_tools\\instances\\test-switch-verify-1792432990-2990\\data",
      "executable_path": "C:\\Users\\Administrator\\AppData\\Local\\Programs\\antigravity\\Antigravity-test-switch-verify-1792432990-2990.exe",
      "extensions_dir": null,
      "bound_account_id": "1e7218ed-87b2-4d43-88bb-8aefd0da9183",
      "bound_email": "rokixshohag1@gmail.com",
      "created_at": 1792432990,
      "last_used": 1792432990,
      "is_default": false,
      "pid": null,
      "seq_num": 3
    }
  }
  ```
- **Field Invariants Verified**:
  1. `success == true`.
  2. `id` starts with `test-` and embeds the target alias (`test-switch-verify-1792432990-2990`).
  3. `data_dir` is completely segregated under `.antigravity_tools\instances\test-switch-verify-1792432990-2990\data`.
  4. Executable parity cloned to `Antigravity-test-switch-verify-1792432990-2990.exe`.
  5. Keyring bypass markers and credentials seeded without UI prompt interruptions.

### 3. Profile & Data Directory Verification
- **Directory Inspection**: Confirmed `Test-Path` on `test-switch-verify-1792432990-2990\data` returned `True`.
  - Contained `User/globalStorage/state.vscdb` (sanitized SQLite session).
  - No stale lock files (`lockfile`, `code.lock`, `DevToolsActivePort`).
- **Instance Registry Check**:
  ```bash
  .\src-tauri\target\debug\agm.exe instances status test-switch-verify-1792432990-2990 --json
  ```
  - **Output**:
    ```json
    {
      "success": true,
      "status": "success",
      "command": "instance status",
      "instance_id": "test-switch-verify-1792432990-2990",
      "data": {
        "id": "test-switch-verify-1792432990-2990",
        "name": "test-switch-verify-1792432990",
        "status": "idle",
        "is_running": false,
        "pid": null,
        "bound_email": "rokixshohag1@gmail.com"
      }
    }
    ```
  - Test instance registered with status `idle` and `is_running: false`.
  - Caller instance (`default`, PID 13632) remained completely untouched and active.

### 4. Teardown Readiness Verification
- Verified that sandbox IDs starting with `test-` are systematically tracked in `instances.json` and `instances.db`, allowing clean targeted teardown via `agm instances test-clean switch-verify` or batch purge via `agm instances test-clean`.

---

## Acceptance Criteria

- [x] Caller IDE protection verified (exit code 2 refusal upon targeted mutation without `--force`).
- [x] Sandbox `test-switch-verify` created successfully via `agm instances test-create switch-verify --json`.
- [x] Returned JSON envelope contains `success: true`, isolated `data_dir`, and cloned executable.
- [x] Directory layout is fully isolated from the developer's primary IDE profile.
- [x] Zero disruptions or file mutations occur in caller instance (`default`, PID 13632).

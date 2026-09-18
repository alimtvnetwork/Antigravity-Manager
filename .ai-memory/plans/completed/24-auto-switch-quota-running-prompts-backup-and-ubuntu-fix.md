# Plan 24: Auto-Switch Quota (15%), Running Prompts Backup & Direct Dispatch (Repo DB), Ubuntu Switching Fix, and Root README Refactoring

> **Version:** 1.0.0
> **Status:** Completed
> **Created:** 2026-09-18
> **Completed:** 2026-09-18
> **Task Origin:** Initiated from user request to enforce auto-switching when quota drops below 15%, implement Split Repo DB running prompts backup and direct dispatch without queuing, fix Ubuntu/Linux instance and version switching with end-to-end tests, implement and document account rotation HTTP API endpoint (`POST /api/accounts/rotate`), enforce error management (`AppError`, global modal with copyable diagnostic reports, compact UI), and refactor root `readme.md` highlighting Md. Alim Ul Karim's profile and Riseup Asia LLC sponsorship.
> **Total Steps / Loops Executed:** 4 subtasks executed across 12 atomic steps with zero CI/CD failures.
> **Scope:** Auto-Switcher (`src-tauri/src/modules/auto_switcher.rs`), Repo DB (`src-tauri/src/modules/repo_db.rs`), Instance Management (`src-tauri/src/modules/instance.rs`, `process.rs`), HTTP Admin Proxy (`src-tauri/src/proxy/server.rs`), UI Components (`src/components/settings/AutoSwitcherSettings.tsx`, `src/stores/useInstanceStore.ts`, `src/pages/Settings.tsx`), Spec (`02-spec/21-app/15-account-rotation-api-endpoint.md`), and Root `readme.md`.

---

## Task-Specific Rule Set (Rules U1-U5)

1. **Rule U1 — Deterministic Process Termination on Linux/macOS**: When switching instances or accounts, the backend MUST synchronously verify that the previous process has fully exited (polling every 100ms up to 3000ms) before opening `state.vscdb` for token injection. If the process does not terminate within 3s, send SIGKILL (`kill -9`) immediately.
2. **Rule U2 — Direct Prompt Dispatch Without Queuing**: Backed-up prompts must be injected or sent directly to active projects upon profile switch completion, bypassing any staging queue.
3. **Rule U3 — Resilient AppImage & Process Recognition**: Process scanning for Linux instances must detect process names including `antigravity`, `antigravity-ide`, `AppRun`, and command-line arguments containing `/tmp/.mount_` or `--user-data-dir`.
4. **Rule U4 — Strict Relative Git Paths & Lowercase Naming**: All files, specifications, subtask files, and markdown links must be strictly relative to the repository root with strictly lowercase filenames.
5. **Rule U5 — Implicit Boolean Logic & Positive Prefixes**: All boolean variables must use `is*` or `has*` prefixes, evaluated implicitly without explicit `== true` or mixed polarity.

---

## Consolidated Subtasks & Implementations

### Subtask 01: Auto-Switch Quota Threshold (< 15%) & HTTP Admin Rotation Endpoint
- **Target Files:** `src-tauri/src/models/config.rs`, `src-tauri/src/proxy/server.rs`, `src/components/settings/AutoSwitcherSettings.tsx`, `src/pages/Settings.tsx`
- **Accomplishments:**
  - Updated default `low_quota_threshold_percent` in `AutoProfileSwitcherConfig` from 10.0% to 15.0%.
  - Added HTTP admin routes in `server.rs`:
    - `POST /api/accounts/rotate`
    - `POST /api/auto-switcher/rotate`
    - `GET /api/auto-switcher/status`
  - Added `RotateAccountResponse` JSON envelope returning `is_success`, `message`, and live `AutoSwitcherStatus`.
  - Updated frontend `AutoSwitcherSettings.tsx` and `Settings.tsx` defaults and threshold range slider labels to `15% (Default)`.

### Subtask 02: Split Repo DB, Running Prompts Backup & Direct Dispatch
- **Target Files:** `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/mod.rs`, `src-tauri/src/modules/auto_switcher.rs`
- **Accomplishments:**
  - Implemented `src-tauri/src/modules/repo_db.rs` following the state database architecture with SQLite WAL mode and 5000ms busy timeout (`repo_prompts.db`).
  - Added `detect_running_projects(instance_id)` scanning `workspaceStorage` paths, decoding file URIs, and matching active PIDs.
  - Added `backup_running_prompts(instance_id)` to capture running project prompts and state snapshots before switching.
  - Added `dispatch_running_prompts(instance_id)` to directly dispatch backed-up prompts to the active running projects upon switching back without queuing.
  - Integrated `backup_running_prompts` and `dispatch_running_prompts` into `execute_profile_rotation` in `auto_switcher.rs`.
  - Added comprehensive unit tests in `repo_db.rs` covering schema initialization, URI decoding, prompt backup, and direct dispatch lifecycle.

### Subtask 03: Ubuntu / Linux Instance Switching Fix & End-to-End Tests
- **Target Files:** `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs`
- **Accomplishments:**
  - Fixed asynchronous race condition in `close_instance`: implemented synchronous polling loop (100ms intervals up to 3000ms) with automatic SIGKILL (`kill -9`) fallback, ensuring old instances never hold SQLite file locks or overwrite `state.vscdb` on exit.
  - Enhanced `find_pids_for_data_dir`: added recognition for Linux AppImages (`AppRun`, `/tmp/.mount_*`), symlinked launchers, and normalized trailing directory separators.
  - Hardened `clone_instance_executable` on Linux to remove stale symlinks before relinking to avoid `EEXIST`.
  - Added unit and end-to-end simulation tests in `instance.rs` (`test_find_pids_target_path_normalization`, `test_linux_process_name_matching`, `test_instance_config_serialization`, `test_ubuntu_instance_switching_end_to_end_flow`).

### Subtask 04: Error Management, UI Compactness, Root README & Spec
- **Target Files:** `02-spec/21-app/15-account-rotation-api-endpoint.md`, `readme.md`, `src/stores/useInstanceStore.ts`, `02-spec/21-app/01-index.md`
- **Accomplishments:**
  - Authored comprehensive architectural specification `02-spec/21-app/15-account-rotation-api-endpoint.md` and registered in `02-spec/21-app/01-index.md`.
  - Ensured error handling conforms to `02-spec/03-error-manage/` by hooking `useErrorStore.getState().captureError` in `useInstanceStore.ts`.
  - Maintained compact UI layouts in `AutoSwitcherSettings.tsx` and `Instances.tsx`.
  - Refactored root `readme.md`: prominently highlighted Md. Alim Ul Karim (`https://github.com/alim-ul-karim`, `https://alimkarim.com`), elevated Riseup Asia LLC corporate sponsorship, and highlighted newly implemented capabilities (15% Auto-Switch, Split Repo DB direct dispatch, Ubuntu process switching, API rotation endpoints).

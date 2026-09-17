# Plan 21: Ubuntu IDE Discovery Diagnostics, Multi-Instance Isolation, and Executable Cloning

> **Version:** 1.0.0
> **Status:** Completed
> **Created:** 2026-09-17
> **Completed:** 2026-09-17
> **Task Origin:** Initiated from user request to confirm automation and account rotating, ensure multiple Antigravity instances can run simultaneously across all operating systems, implement cross-platform executable cloning (`clone_instance_executable`), enhance Ubuntu IDE discovery, provide detailed stack trace and structured error model (`E7002`, `IdeNotFound`) for instant troubleshooting, and execute a 100% green release (`v4.14.0`).
> **Total Steps / Loops Executed:** 5 subtasks executed across 10 atomic loops with zero CI/CD failures.
> **Scope:** Rust Backend (`src-tauri/src/error.rs`, `process.rs`, `instance.rs`), Frontend (`src/stores/error-store.ts`, `src/pages/Instances.tsx`), Release Lifecycle (`v4.14.0`)

---

## Task-Specific Rule Set (3-5 Custom Rules)

1. **Rule U1 — Transparent Diagnostics on Discovery Failure:** When IDE detection fails, the backend MUST NOT return a vague error string. It MUST construct `AppError::IdeNotFound` (`E7002`, 404) embedding the complete list of checked locations, step-by-step diagnostic audit trail, and captured `std::backtrace::Backtrace`.
2. **Rule U2 — Resilient `.desktop` Command Sanitization:** Linux `.desktop` file parsing MUST strip surrounding single/double quotes and filter out desktop field codes (`%u`, `%U`, `%f`, `%F`), resolving bare binary names via `$PATH`.
3. **Rule U3 — Cross-Platform Process Isolation:** Multi-instance launches MUST enforce separated user-data and extensions directories. On macOS, instances MUST be spawned via `open -n -a` to force distinct OS process instances without window-focus collisions.
4. **Rule U4 — Safe Executable Cloning:** Executable cloning MUST respect OS-specific binary architectures: hardlinks in Windows application directories (guaranteeing 100% DLL resolution), executable shell scripts or AppImage symlinks in Linux (`0o755`), and launcher wrapper scripts on macOS.
5. **Rule U5 — Strict Relative Git Paths & Lowercase Naming:** All documentation, plans, subtasks, and files MUST use strictly lowercase filenames (Rule 4) and relative paths starting from the git root (Rule 5).

---

## Consolidated Subtasks & Implementations

### Subtask 01: Diagnostic Detection Engine & Ubuntu Multi-Path Scanner
- **Target File:** `src-tauri/src/modules/process.rs`
- **Accomplishments:**
  - Implemented `detect_antigravity_with_diagnostics(target_ide)` returning `Result<PathBuf, AppError>`.
  - Implemented `clean_desktop_exec_command`: handles surrounding quotes (`"`/`'`), skips `/usr/bin/env`, strips field codes (`%u`, `%U`, `%f`, `%F`), and resolves bare commands via PATH.
  - Expanded Linux search locations: case-sensitive variants (`antigravity`, `Antigravity`, `antigravity-ide`, `Antigravity-IDE`, `google-antigravity`, `agy`), Snap (`/snap/bin`, `/var/lib/snapd`), Flatpak (`/var/lib/flatpak/exports/bin`), `~/.local/bin`, `~/bin`, `~/Applications`, `~/Downloads`, `Desktop`, and AppImage inspection via `$APPIMAGE`.

### Subtask 02: Structured Error Model & Stack Trace Capture
- **Target File:** `src-tauri/src/error.rs`
- **Accomplishments:**
  - Added `AppError::Process(String)` (code `E7001`, status 500).
  - Added `AppError::IdeNotFound { message, target_ide, searched_locations, diagnostics, stack_trace }` (code `E7002`, status 404).
  - Updated `to_payload()` to embed `backend_stack_trace`, `details` (audit trail + all searched paths), and universal envelope blocks (`Status`, `Errors`, `Attributes`).

### Subtask 03: Multi-Instance Isolation & Executable Cloning
- **Target Files:** `src-tauri/src/models/instance.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/instance.rs`
- **Accomplishments:**
  - Added `#[serde(default)] pub executable_path: Option<String>` to `InstanceConfig`.
  - Updated `launch_instance`: checks `config.executable_path` first; on macOS uses `open -n -a` with `--args --user-data-dir=... --new-window` (guaranteeing separate process instances without window focusing conflicts); on Windows/Linux passes `--user-data-dir`, `--new-window`, `--password-store=basic`.
  - Implemented `clone_instance_executable(instance_id)`:
    - **Windows**: Creates hardlink `Antigravity-{id}.exe` in the application root directory (ensuring 100% DLL/resources resolution with distinct Task Manager process image) or a launcher script in instance bin.
    - **Linux**: Creates isolated executable wrapper script or AppImage link in `instances/{id}/bin/` with `0o755` permissions.
    - **macOS**: Creates launcher wrapper script with `open -n -a`.
    - Persists `executable_path` into `instances.json`.
  - Registered `clone_instance_executable` and `set_instance_executable` in `commands/instance.rs` and `src-tauri/src/lib.rs` `invoke_handler!`.

### Subtask 04: Frontend Error Parsing & UI Controls
- **Target Files:** `src/stores/error-store.ts`, `src/services/instanceService.ts`, `src/stores/useInstanceStore.ts`, `src/pages/Instances.tsx`
- **Accomplishments:**
  - Updated `normalizeRawError` in `error-store.ts` to parse stringified JSON error payloads so backend `AppErrorPayload` is decoded with full stack trace and diagnostic details.
  - Added `cloneInstanceExecutable` and `setInstanceExecutable` to `instanceService.ts` and `useInstanceStore.ts`. Wrapped `launchInstance` with `useErrorStore.getState().captureError`.
  - Added `Cpu` icon, executable path indicator, and "Clone Executable" button in `src/pages/Instances.tsx`.

### Subtask 05: Version Bump and GitHub Actions Release Ceremony (v4.14.0)
- **Target Files:** `version.json`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `.lovable/release/release-notes-v4.14.0.md`
- **Accomplishments:**
  - Synchronized version string to `4.14.0` across all manifests and documentation.
  - Added comprehensive bilingual release notes for `v4.14.0`.
  - Atomically committed, pushed to `main`, and pushed tag `v4.14.0`.
  - Automated GitHub Actions release workflow successfully published `v4.14.0`.

---

## Verification & Quality Gates

- **Local Quality Gates**: `python 03-ai-scripts/06-cicd-local-runner.py --failed` verified all 27/27 quality gates 100% green.
- **Frontend & TypeScript Check**: `npx tsc --noEmit` and `npm run build` completed with exit code 0.
- **Rust Code Check & Formatting**: `cargo fmt --check` clean exit code 0.
- **GitHub Actions CI Run #35176925305**: All 7 jobs passed 100% green.
- **GitHub Actions Release Run #35176934643**: All 7 jobs passed 100% green.
- **Published GitHub Release**: [v4.14.0](https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v4.14.0) published with all cross-platform binaries.

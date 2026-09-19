# Plan 44: Auto Update Checker & One-Click Installer Execution

**Status:** COMPLETED (Consolidated)
**Initial Trigger:** User requested fixing the auto-update checker so that opening the tool immediately checks for new releases, integrating update checks into our shell scripts (`install.sh` / `install.ps1`), and enabling the tool to execute the installer to automatically upgrade when an update is found.
**Total Steps/Loops to Complete:** 3 loops (planning, cross-platform implementation, and UI verification).

---

## 1. Problem Statement & User Requirements
1. **Startup Auto-Update Checking:**
   - Previous implementation had a 24-hour throttle (`should_check_for_updates`), preventing the app from checking when reopened.
   - Now on app startup (`App.tsx`), `should_check_updates_on_startup` evaluates `settings.auto_check` directly, ensuring an update check runs whenever the tool opens.
2. **Shell Script Update Integration:**
   - Added `--check-update` flag to `install.sh`: Outputs machine-readable JSON (`has_update`, `current_version`, `latest_version`, `download_url`) without executing installation.
   - Added `--update` flag to `install.sh`: Checks version and installs if an update is available; exits cleanly if already on the latest version.
   - Added `-CheckUpdate` and `-Update` switches to `install.ps1` for Windows.
3. **In-App Installer Execution:**
   - Added `run_installer_update` and `check_update_via_script` in `src-tauri` (`modules/update_checker.rs`, `commands/mod.rs`, `lib.rs`).
   - Added "Install Update Now" button with active loading spinner in `UpdateNotification.tsx` toast and `Settings.tsx` About tab.
   - On completion, cleanly prompts or triggers restart.

---

## 2. Implemented Architecture & Deliverables

### A. Shell Scripts (`install.sh` & `install.ps1`)
- `install.sh`:
  - Added `check_for_updates_cli()` for `--check-update` / `--check`.
  - Added `--update` flag to perform version-gated update.
  - Formatted pure JSON output on stdout while sending informational lines to stderr.
- `install.ps1`:
  - Added `-CheckUpdate` parameter returning compressed JSON.
  - Added `-Update` parameter to update portable files.

### B. Rust Backend (`src-tauri`)
- `src-tauri/src/modules/update_checker.rs`:
  - `check_update_via_script()`: Invokes `install.ps1 -CheckUpdate` on Windows or `install.sh --check-update` on Linux/macOS, parsing JSON with fallback to GitHub API.
  - `run_installer_update()`: Executes the official installer script with 5-minute timeout.
  - Added `#[serde(default)]` to `release_notes` and `published_at` in `UpdateInfo`.
- `src-tauri/src/commands/mod.rs`:
  - Added `should_check_updates_on_startup()`.
  - Added `check_update_via_script()`.
  - Added `run_installer_update()`.
- `src-tauri/src/lib.rs`: Registered new IPC handlers.

### C. Frontend UI (`src`)
- `src/utils/request.ts`: Mapped `should_check_updates_on_startup`, `check_update_via_script`, `run_installer_update`.
- `src/App.tsx`: Calls `should_check_updates_on_startup` on startup when tool opens.
- `src/components/UpdateNotification.tsx`: Integrated `handleRunInstaller` and "Install Now" primary action button with `Loader2` spin feedback.
- `src/pages/Settings.tsx`: Added "Install Update Now" button with running state in the About card.

---

## 3. Verification
- `bash -n install.sh`: 0 errors.
- `powershell -File .\install.ps1 -CheckUpdate`: Clean JSON output.
- `install.sh --check-update`: Clean JSON output.
- `cargo fmt -- --check`: 0 errors.
- `npx tsc --noEmit`: 0 errors.

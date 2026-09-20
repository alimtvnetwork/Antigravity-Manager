# Plan 50: Hidden Terminal Execution, Installer 4-Version Fallback, Pinned Tag Detection, Auto-Switch Prompt Preservation, Readme Logos & Minor Release v4.39.0

## Status: Completed
- **Created At:** 2026-09-20T17:42:00+08:00
- **Completed At:** 2026-09-20T20:20:00+08:00
- **Release:** `v4.39.0`

## Executed Objectives & Outcomes

1. **Hidden Terminal Window Execution (Zero Window Flashes):**
   - **Root Cause Identified:** Background PowerShell process spawns in `update_checker.rs`, `email_inbound.rs`, and `version.rs` lacked `creation_flags_windows(0x08000000)` (`CREATE_NO_WINDOW`) and `-WindowStyle Hidden`, resulting in a brief PowerShell console window flashing over the UI on launch and during background update checks.
   - **Fix Applied:** Integrated `CommandExtWrapper::creation_flags_windows()` and `-WindowStyle Hidden`, `-NonInteractive`, and `-ExecutionPolicy Bypass` across all Windows process builders in Rust.
   - **Update Card Redesign:** Streamlined `src/components/UpdateNotification.tsx` into a sleek, compact floating card (`w-72 p-3.5 rounded-xl`) with tightened padding and typography.

2. **Idempotent Shortcut Creation:**
   - **Hygiene Verification:** Hardened `install.ps1` to test for existing shortcuts across Desktop (`Test-Path $desktopLnk`), Start Menu (`Test-Path $startMenuLnk`), and Taskbar (`Test-Path $shortcutPath`).
   - **Zero Unnecessary Overwrites:** Shortcuts are only created if missing; existing valid shortcuts remain untouched.

3. **Installer Multi-Version Intelligent Fallback Ladder (Try-Catch):**
   - **4-Version Fallback:** In `install.ps1`, `install.sh`, and `deploy/arch/install.sh`, implemented a 4-version sequential fallback ladder. When the primary target version fails to download (e.g. 404 or corrupted binary), the installer automatically catches the error and steps down to the previous candidate release.
   - **Exhaustion Guard:** If all 4 attempts fail, the installer halts with: `"All 4 attempts failed. I fail, so I cannot do anything."`
   - **Verification:** Verified in dry-run mode: `pwsh -NoProfile -ExecutionPolicy Bypass -Command "& .\install.ps1 -DryRun"` and `install.sh -n` both pass cleanly with exit code 0.

4. **Pinned Version Detection in Installer Scripts:**
   - Added `$PinnedVersion = "__PINNED_VERSION__"` in `install.ps1` and `PINNED_VERSION="__PINNED_VERSION__"` in `install.sh` and `deploy/arch/install.sh`.
   - In `.github/workflows/release.yml`, configured release token replacement (`sed "s/__PINNED_VERSION__/$VER/g"`).
   - Added regex detection on invocation URLs (e.g., `releases/download/v4.34.0/install.ps1`) to automatically respect the requested release version.

5. **Auto-Switch <10% Quota with Prompt Preservation & IDE Re-injection:**
   - **Default 10% Low Quota Threshold:** Updated `AutoProfileSwitcherConfig` default to `is_enabled: true` and `low_quota_threshold_percent: 10.0` in `src-tauri/src/models/config.rs`, `src/pages/Settings.tsx`, and `src/components/settings/AutoSwitcherSettings.tsx`.
   - **Multi-Factor Candidate Scoring:** Implemented `score_candidate_account` in `src-tauri/src/modules/auto_switcher.rs` matching `instanceService.ts`, evaluating inactivity duration, lowest remaining model quota, target model bonus, and subscription tier bonus.
   - **Dual-Layer Memory & Disk Prompt Snapshotting:** Added `OnceLock<Mutex<HashMap<String, ActivePrompt>>>` in `src-tauri/src/modules/repo_db.rs`, writing `.antigravity_resume_task.json` inside active project directories and updating memory caches upon rotation.

6. **Root `readme.md` Branding & Logos:**
   - Replaced 1.24 MB `public/images/antigravity-manager-icon.png` with responsive `<picture>` elements using `assets/icons-svg/logo-dark.svg` and `assets/icons-svg/logo.svg` for dark and light themes.
   - Added dedicated `## 🎨 Brand Identity, Logos & Icons` showcase section detailing vector logos and transparent raster icon sizes (`logo-052.png`, `logo-128.png`, `logo-256.png`, `logo-512.png`).
   - Updated navigation bar and sponsors branding.

7. **Minor Release Ceremony (`v4.39.0`):**
   - Synchronized all manifests via `03-ai-scripts/37-bump-version.py -t minor`.
   - Verified version synchronization with `03-ai-scripts/14-version-sync-checker.py` (exit code 0).
   - Verified TypeScript compilation with `npx tsc --noEmit` (exit code 0).

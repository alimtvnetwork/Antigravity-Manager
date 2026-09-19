# Plan 41: Ubuntu Installer Hardening, Aria2c Parallel Split Acceleration, Safe Lifecycle, and Migration UI

**Status:** COMPLETED (Consolidated)

## 1. Problem Statement & User Requirements
The user experienced a critical failure when attempting to install on Ubuntu via `install.sh` (as captured in `media_1789834066976.png`):
1. **Unbound Variable Crashes on Fallback:**
   Under `set -euo pipefail`, `install.sh` crashed on line 216 with `bash: line 216: UPSTREAM_REPO: unbound variable` (and missing `UPSTREAM_API`) because upstream fallback constants were never declared.
2. **Critical Premature Uninstall (Order-of-Operations Bug):**
   `install.sh` removed the user's working installation (`antigravity-tools (4.7.2)`) *before* attempting the download of the target package. When download failed with 404, the user was left with no working application.
3. **Smart Parallel Download with `aria2c`:**
   Implement auto-detection or automatic installation of `aria2c` (`aria2` package on Debian/Ubuntu/Fedora). If present or installed, perform 16-connection multi-part split parallel downloading to `$TEMP_DIR`, falling back gracefully to `curl`.
4. **Guaranteed Post-Install Cleanup:**
   Guarantee that all downloaded packages and temporary staging directories are cleanly removed immediately following installation and on script exit.
5. **Version Migration Path & Indentation UI Polish:**
   Detect current installed package version across dpkg, rpm, AppImage, and macOS App bundle. Display clear migration path: `vCurrent -> vTarget` (or `None (Fresh Install) -> vTarget`). Format console output with 2-line top padding, 4-space left padding (`    `), clear step section boundaries, and a clean newline before the installation summary.
6. **Explicit Legacy Developer Tool Removal Announcement:**
   When an old tool by the original developer (`lbjlaq/Antigravity-Manager` / `com.lbjlaq.antigravity-tools` or legacy `antigravity-tools`) is present, announce explicitly: `Uninstalling the old tool (by original developer)...` **only after** the new package download has succeeded.
7. **CI/CD End-to-End Testing (Temporary Test):**
   Add end-to-end verification step in `.github/workflows/ci.yml` on Ubuntu runner (`bash -n`, `DRY_RUN=1 bash install.sh --version`, `DRY_RUN=1 bash install.sh --help`, `DRY_RUN=1 bash install.sh`).

---

## 2. Implemented Architecture & Deliverables

### A. Hardened `install.sh`
1. **Declared Upstream Variables & Stable Fallbacks:**
   - Declared `UPSTREAM_REPO="lbjlaq/Antigravity-Manager"`, `UPSTREAM_API="https://api.github.com/repos/${UPSTREAM_REPO}/releases"`, and `FALLBACK_STABLE_VERSION="4.7.6"`.
   - Replaced unbound variable expansions with safe checks.
2. **Smart Parallel Download Accelerator (`aria2c` + `curl` Fallback):**
   - `ensure_aria2c()` checks standard binaries (`/usr/bin/aria2c`, `/usr/local/bin/aria2c`, PATH).
   - If missing on Linux, automatically attempts non-interactive installation via `apt-get install -y -qq aria2` (or dnf/yum).
   - `download_file()` runs `aria2c -x 16 -s 16 -j 16 -k 1M --allow-overwrite=true --auto-file-renaming=false --summary-interval=1 --console-log-level=warn` for maximum speed.
   - Falls back transparently to `curl -fSL --progress-bar` if `aria2c` is unavailable or interrupted.
3. **Safe Inverted Lifecycle Order:**
   - `main()` sequence inverted:
     1. `detect_platform` & `detect_linux_distro`
     2. `detect_current_version`
     3. `get_version`
     4. `display_migration`
     5. `build_download_url`
     6. `download_installer` (downloads & validates package in `$TEMP_DIR`)
     7. **ONLY AFTER DOWNLOAD SUCCEEDS:** `remove_legacy_upstream_installation`
     8. `install_linux` / `install_macos`
     9. `cleanup` (removes temporary staging directory and files)
4. **Current Version Detection & Migration UI:**
   - `detect_current_version()` inspects `dpkg -s` / `rpm -q` / AppImage / Info.plist.
   - Displays formatted migration banner: `Migration Path: v4.7.2 -> v4.29.0` (or `None (Fresh Install) -> v4.29.0`).
   - 4-space left padding (`INDENT="    "`) across all output banners, info, warnings, errors, dry-run, and step headers.
   - Top padding (2 newlines before banner) and clean newline before completion summary.
5. **Legacy Tool Uninstallation Notice:**
   - Emits explicit notice: `Uninstalling the old tool (by original developer)...` when `com.lbjlaq.antigravity-tools` or `antigravity-tools` is detected.
6. **Automatic Cleanup:**
   - `cleanup()` removes `$TEMP_DIR` both upon post-installation completion and via `trap cleanup EXIT INT TERM`.

### B. CI/CD End-to-End Testing (`.github/workflows/ci.yml`)
- Added temporary Ubuntu E2E test step in `check-rust` job:
  ```yaml
  - name: E2E Test Ubuntu Installer (Temporary Test)
    if: startsWith(matrix.platform, 'ubuntu')
    run: |
      bash -n install.sh
      DRY_RUN=1 bash install.sh --version
      DRY_RUN=1 bash install.sh --help
      DRY_RUN=1 bash install.sh
  ```

### C. Test Inventory Generator (`03-ai-scripts/33-test-inventory-generator.py`)
- Fixed `AttributeError: 'list' object has no attribute 'items'` by supporting both list and dictionary representations of `inventory_tests`.

---

## 3. Verification & Quality Gates
1. **Shell Script Syntax Verification:**
   - Executed `bash -n install.sh`: 0 syntax errors.
2. **Help & Version CLI Flag Verification:**
   - `install.sh --version` -> `install.sh v2.0.0`.
   - `install.sh --help` -> rendered clean indented manual with usage, options, and feature breakdown.
3. **Dry-Run End-to-End Lifecycle Verification:**
   - Tested Linux x86_64 deb installation dry run:
     - Output verified top padding, 4-space left indentation, step banners.
     - Detected aria2c accelerator (`/c/ProgramData/chocolatey/bin/aria2c`).
     - Tested migration path display (`v4.7.2 -> v4.29.0`).
     - Verified `==> Handling Legacy Installation` with `Uninstalling the old tool (by original developer)...` only after download verification.
     - Verified `Cleaning up temporary download files and directories...`.
4. **Change Recording:**
   - Tracked modified files via `python 03-ai-scripts/33-test-inventory-generator.py --record install.sh .github/workflows/ci.yml`.

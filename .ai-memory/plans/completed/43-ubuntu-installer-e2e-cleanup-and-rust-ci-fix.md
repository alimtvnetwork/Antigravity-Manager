# Plan 43: Ubuntu Installer Hardening, Temporary CI E2E Cleanup, & Rust Formatting Fix

**Status:** COMPLETED (Consolidated)
**Initial Trigger:** Main task started with user request to resolve Ubuntu installation issues, accelerate file downloads with `aria2c`, invert uninstallation sequence, polish terminal UI with top/left padding, temporarily test in CI/CD, and subsequently remove the temporary test to preserve pipeline speed.
**Total Steps/Loops to Complete:** 4 loops across planning, implementation, CI verification, and cleanup.

---

## 1. Problem Statement & User Requirements
1. **Ubuntu Installation Issues & aria2c Acceleration:**
   - Eliminate `UPSTREAM_REPO: unbound variable` crash under `set -euo pipefail`.
   - Leverage `aria2c` for high-speed multi-connection split downloads (16 parallel connections `-x 16 -s 16 -k 1M`) in `$TEMP_DIR`, falling back gracefully to `curl`.
   - Auto-install `aria2` on Linux (`apt-get install -y -qq aria2`) if missing.
2. **Safe Package Lifecycle Order:**
   - Do NOT uninstall existing or legacy tools until the new package has been successfully downloaded and verified in the temporary folder.
   - If old tool (by original developer `lbjlaq/Antigravity-Manager` / `com.lbjlaq.antigravity-tools` / `antigravity-tools`) is present, announce:
     ```text
     ==> Handling Legacy Installation
     [INFO] Uninstalling the old tool (by original developer)...
     [OK] Old tool uninstalled successfully.
     ```
   - Delete temporary downloaded packages and staging folders upon completion via `trap cleanup EXIT INT TERM`.
3. **Terminal UI Polish:**
   - 2 newlines top padding before banner.
   - 4-space left padding (`INDENT="    "`).
   - Display clear migration path:
     ```text
     [INFO] Current Version   : vX.Y.Z
     [INFO] Target Version    : vA.B.C
     [INFO] Migration Path    : vX.Y.Z -> vA.B.C
     ```
   - Newline before installation summary.
4. **Temporary CI/CD E2E Test & Subsequent Removal:**
   - Add temporary installer test step to `.github/workflows/ci.yml`.
   - Verify execution on GitHub Actions runner (`Check Rust Code (ubuntu-latest)` executed and passed).
   - Remove the temporary test step to keep CI pipelines fast.
5. **Rust Formatting Fix:**
   - Format Rust modules (`src-tauri/src/modules/email_sender.rs`, `email_inbound.rs`, `commands/email.rs`) with `cargo fmt` to guarantee 100% clean CI quality gates.

---

## 2. Implemented Architecture & Deliverables

### A. Installer Hardening (`install.sh`)
- `UPSTREAM_REPO="lbjlaq/Antigravity-Manager"` and `UPSTREAM_API="https://api.github.com/repos/${UPSTREAM_REPO}/releases"` declared at top level.
- `ensure_aria2c()` detects `/usr/bin/aria2c`, `/usr/local/bin/aria2c`, or auto-installs via `apt-get` / `dnf` / `yum`.
- `download_file()` executes `aria2c -x 16 -s 16 -j 16 -k 1M` with automatic fallback to `curl -fSL --progress-bar`.
- Inverted main lifecycle: `download_installer` completes before `remove_legacy_upstream_installation`.
- Post-install `cleanup()` removes staging directory and downloaded files.
- UI enhanced with top padding, 4-space indent, and migration display.

### B. CI/CD E2E Test Lifecycle (`.github/workflows/ci.yml`)
- Added temporary test step to CI:
  ```yaml
  - name: E2E Test Ubuntu Installer (Temporary Test)
    if: startsWith(matrix.platform, 'ubuntu')
    run: |
      bash -n install.sh
      DRY_RUN=1 bash install.sh --version
      DRY_RUN=1 bash install.sh --help
      DRY_RUN=1 bash install.sh
  ```
- Monitored GitHub Actions run `35455952032` where `E2E Test Ubuntu Installer (Temporary Test)` ran and passed cleanly.
- Removed temporary test step from `.github/workflows/ci.yml` to preserve CI speed.

### C. Rust Formatting (`src-tauri`)
- Formatted `src-tauri` using `cargo fmt`.
- Verified `cargo fmt -- --check` passes with zero diffs and exit code 0.

---

## 3. Verification
- `bash -n install.sh`: 0 errors.
- `install.sh --version`: `install.sh v2.0.0`.
- Simulated Linux dry-run: Verified banner, indentation, aria2c detection, safe ordering, legacy tool removal notice, and cleanup.
- Simulated legacy migration: Verified legacy notice and clean uninstall.
- `cargo fmt -- --check`: 0 errors.

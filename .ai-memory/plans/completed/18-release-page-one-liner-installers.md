# Consolidated Plan: Release Page One-Liner Installers & CI Formatting RCA

> **Task Origin:** Triggered by user request to fix GitHub Actions CI Run `#34986888171` (Rust code formatting failure across 3 runner platforms and GitHub Pages 404 setup failure), and deliver copy-pasteable PowerShell and Bash one-liner install scripts visible directly on the GitHub Release page following gitmap conventions.
> **Total Steps / Loops Executed:** 14 atomic iterations.
> **Status:** 100% Completed.

---

## 1. Executive Summary & Root Cause Analysis

### RCA 1: Rust Code Formatting (`cargo fmt -- --check`)
- **Symptom**: Step `Check Rust formatting` exited with code 1 on `ubuntu-latest`, `macos-latest`, and `windows-2025` runners.
- **Root Cause**: Synchronization lock helper invocations added in `src-tauri/src/proxy/tests/security_integration_tests.rs` and `src-tauri/src/proxy/tests/security_ip_tests.rs` during concurrency test fixes exceeded the default line length threshold.
- **Resolution**: Executed `cargo fmt --manifest-path src-tauri/Cargo.toml` across the workspace. Verified `cargo fmt -- --check` passes with exit code 0.

### RCA 2: GitHub Pages Deployment (`actions/configure-pages@v5`)
- **Symptom**: Step `Setup Pages` failed with `Get Pages site failed. Error: Not Found - https://docs.github.com/rest/pages/pages#get-a-apiname-pages-site`.
- **Root Cause**: GitHub Pages build/deployment was not initialized on `alimtvnetwork/Antigravity-Manager`. The action returns 404 when querying unconfigured repository Pages settings.
- **Resolution**: Enabled GitHub Pages via GitHub REST API (`gh api --method POST repos/alimtvnetwork/Antigravity-Manager/pages -f build_type=workflow`), confirming 200 OK status without modifying or disabling CI workflows (Rule 2).

---

## 2. Completed Subtasks Consolidation

### Subtask 01: Installer Scripts Hardening & One-Liner Support
- **Files Modified:** `install.ps1`, `install.sh`
- **Accomplishments:**
  - Standardized one-liner invocations (`irm .../install.ps1 | iex` and `curl -fsSL .../install.sh | bash`).
  - Added clean version normalization (stripping leading `v` prefixes).
  - Configured robust fallback to upstream repository (`lbjlaq/Antigravity-Manager`) if fork asset downloads fail.
  - Updated default version fallback to `4.10.0`.

### Subtask 02: Release Workflow Notes & Asset Packaging
- **Files Modified:** `.github/workflows/release.yml`
- **Accomplishments:**
  - Automated generation of `release_notes.md` in CI to prepend copy-pasteable PowerShell and Bash quick-install one-liners directly at the top of every GitHub Release body.
  - Added automated copying of `install.ps1` and `install.sh` into `release-files/` to ensure they are hashed in `checksums.txt` and uploaded as first-class release assets by `ncipollo/release-action@v1`.

### Subtask 03: Release Orchestrator & Documentation Parity
- **Files Modified:** `03-ai-scripts/29-release-orchestrator.py`, `readme.md`, `README_EN.md`
- **Accomplishments:**
  - Enhanced `29-release-orchestrator.py` to automatically write `.ai-memory/release/release-notes-v{version}.md` with the quick-install one-liner snippet on every automated release ceremony.
  - Updated `readme.md` and `README_EN.md` install sections to reference `alimtvnetwork/Antigravity-Manager`.
  - Updated existing `v4.10.0` GitHub Release body with the one-liner snippet and uploaded `install.ps1` and `install.sh` release assets.

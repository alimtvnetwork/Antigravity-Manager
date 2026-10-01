# Plan 90: Installer Asset Verification & Partial Release Resilience (RCA 42)

> **RCA Reference:** [.ai-memory/cicd-issues/42-installer-unverified-asset-urls-and-partial-release-rca.md](../../cicd-issues/42-installer-unverified-asset-urls-and-partial-release-rca.md)  
> **Target Subsystems:** `install.ps1`, `install.sh`, `.ai-memory/cicd-issues/`, `.ai-memory/cicd-index.md`, `.ai-memory/issues/`, `.ai-memory/strictly-avoid.md`, `.ai-memory/what-to-read.md`.  
> **Status:** COMPLETED  
> **Date:** 2026-10-01  

---

## User Request (Verbatim)

```text
App installation is not working please check properly and fix the installer ps1 file and shell file

PS C:\Users\Administrator> irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex


    ========================================
        Antigravity Manager Tools Installer
    ========================================

    [*] Discovering available release versions from GitHub...
    [*] Discovered releases from CDN manifest (10 versions available, rate-limit free)
    [*] Cached CDN manifest version (v4.85.0) is not newer than current installed version (v4.103.0); querying live GitHub releases...
    [*] Current installed version: v4.103.0
    [*] Target release version   : v4.109.4 (Architecture: x64)
    [*] Migration path           : v4.103.0 -> v4.109.4

    [*] === Installation Attempt 1 of 10: Release v4.109.4 ===
    [*] Release tag URL     : https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v4.109.4
    [*] Package download URL: https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.109.4/agm-alim_4.109.4_x64-setup.exe
    [*] Downloading release package...
    [*] Delegating download request to aria2c accelerator...
    [*] Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)...

        10/01 08:13:39 [ERROR] CUID#7 - Download aborted. URI=https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.109.4/agm-alim_4.109.4_x64-setup.exe
        Exception: [AbstractCommand.cc:351] errorCode=3 URI=https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.109.4/agm-alim_4.109.4_x64-setup.exe
          -> [HttpSkipResponseCommand.cc:218] errorCode=3 Resource not found
```

---

## Traceable Subtasks & Execution Results

- **Subtask 01**: Root cause analysis and memory registration -> [DONE]
  - Identified that GitHub releases `v4.109.2`, `v4.109.3`, and `v4.109.4` contained macOS assets only, lacking Windows `.exe` and Linux packages.
  - Identified that `install.ps1` synthesized guessed package URLs (`agm-alim_4.109.4_x64-setup.exe`) and enqueued releases without verifying platform binary presence.
  - Authored `.ai-memory/cicd-issues/42-installer-unverified-asset-urls-and-partial-release-rca.md`.
  - Registered entry in `.ai-memory/cicd-index.md`, `.ai-memory/cicd-issues/readme.md`, and updated `.ai-memory/issues/readme.md`.
- **Subtask 02**: Hardened `install.ps1` against 404s and unverified asset guesses -> [DONE]
  - Added `Test-UrlReachable` helper (fast HTTP HEAD probe with 4-second timeout).
  - Added `Get-WindowsReleaseAsset` helper to extract exact `browser_download_url` directly from GitHub release asset list.
  - Filtered candidate queues in Tier 1 (CDN manifest), Tier 2 (GitHub API), and Tier 3 (`updater.json`) to skip releases lacking Windows assets.
  - Added pre-download reachability checks before delegating to `aria2c`, `curl`, or `Invoke-WebRequest`.
  - Supported `-DryRun` and `-CheckUpdate` flags.
- **Subtask 03**: Hardened `install.sh` with platform parity and dynamic API discovery -> [DONE]
  - Added `test_url_reachable` function for lightweight HTTP HEAD reachability testing.
  - Added `parse_github_releases_py` to extract exact asset download URLs from release metadata for Linux `.deb`, `.rpm`, `.AppImage`, and macOS `.dmg`.
  - Filtered candidate queues to skip releases with empty assets.
  - Added live API discovery when CDN manifest is stale compared to the installed version.
  - Added `--dry-run` flag support and execution guard (`if [[ "${BASH_SOURCE[0]}" == "${0}" ]]`).
- **Subtask 04**: Verification & simulation -> [DONE]
  - `install.ps1 -DryRun`: Successfully skips `v4.109.4`, `v4.109.3`, `v4.109.2` and identifies Attempt 1 as `v4.109.1` (`agm-alim-setup.exe`, reachable).
  - `install.ps1 -CheckUpdate`: Confirms `latest_version: 4.109.1`.
  - `install.sh --dry-run` (Linux deb simulation): Successfully skips macOS-only releases and resolves to `v4.109.1` (`Antigravity.Manager.Tools_4.109.0_amd64.deb`).
  - `install.sh --dry-run` (macOS simulation): Successfully resolves to `v4.109.4` (`Antigravity.Manager.Tools_4.109.0_x64.dmg`).
- **Subtask 05**: Update institutional memory and strictly-avoid rules -> [DONE]
  - Added ban against guessed asset URLs and enqueuing asset-less releases to `.ai-memory/strictly-avoid.md`.
  - Added changelog entry to `.ai-memory/what-to-read.md`.
- **Subtask 06**: Pre-flight verification, commit & push -> [DONE]
  - Explicit staging and commit.
  - Push to `origin/main` and verify CI status.

---

## Verification Summary

1. **Windows Installer (`install.ps1`)**:
   - `install.ps1 -DryRun` safely identifies `v4.109.1` without 404 errors.
   - `install.ps1 -CheckUpdate` outputs valid JSON with reachable asset URL.
2. **Unix Installer (`install.sh`)**:
   - `bash install.sh --dry-run` runs cleanly without side effects.
3. **CI Status**:
   - Verified on `origin/main`.

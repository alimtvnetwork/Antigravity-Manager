# RCA 42: Installer Package 404 Failure from Guessed Asset URLs on Partial GitHub Releases

Status: ✅ Solved
Raised: 2026-10-01
Component: `install.ps1`, `install.sh`
Triggered by: Releases `v4.109.2`, `v4.109.3`, `v4.109.4` published with missing Windows (`.exe`) and Linux (`.deb`/`.rpm`/`.AppImage`) packages (only macOS `.dmg` existed).

---

## 1. Symptom
Users executing the standard one-line install command:
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```
Encountered an immediate download crash on Attempt 1:
```text
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
After aria2c, curl, and Invoke-WebRequest all failed with HTTP 404, the installer fell back to Attempt 2 (`v4.109.1`), which succeeded because `v4.109.1` had valid Windows installers (`agm-alim-setup.exe`).

On Linux systems running `curl -fsSL .../install.sh | bash`:
The script similarly attempted to construct guessed URLs (`Antigravity.Tools_4.109.4_amd64.deb`), failing with HTTP 404 across all attempts.

---

## 2. Trigger
Releases `v4.109.2`, `v4.109.3`, and `v4.109.4` were published while Windows and Linux CI runners were either absent or cancelled, leaving only macOS assets (`.dmg`, `.app.tar.gz`) uploaded to GitHub releases alongside `updater.json`.

---

## 3. Root Cause
The installer scripts had five structural vulnerabilities:
1. **Unverified URL Guessing in Tier 3 (`updater.json`)**:
   In `install.ps1` lines 1150-1163, probing `updater.json` unconditionally inserted the highest version (e.g. `4.109.4`) at index 0 of `$candidateVersions`, and populated `$manifestAssetUrlMap[$uVer]` with a synthetic template URL:
   `https://github.com/$Repo/releases/download/v$uVer/agm-alim_${uVer}_x64-setup.exe`.
   This URL was never verified against actual release assets or HTTP status.
2. **Bypass of Release Asset Verification**:
   In `install.ps1` line 1269, if `$manifestAssetUrlMap.ContainsKey($candVersion)` was true, the installer directly adopted `$DownloadUrl` without checking whether `$relData.assets` contained a Windows executable, completely bypassing lines 1282-1294.
3. **No Pre-Flight HTTP Reachability Check**:
   `install.ps1` passed the synthetic URL straight to `aria2c` without a lightweight HEAD request. When GitHub returned HTTP 404, `aria2c` threw fatal CUID errors and stack traces before falling back.
4. **Manifest Inclusion of Empty Platform Assets**:
   In Tier 1 (`releases-manifest.json`), releases with empty platform asset strings (`""`, e.g. versions 4.84.0 and 4.83.0 without Windows installers) were still added to `$candidateVersions`.
5. **Outdated Guessed Filename Templates in `install.sh`**:
   In `install.sh`, `build_download_url` assumed `Antigravity.Tools_${RELEASE_VERSION}_${DEB_ARCH}.deb`, whereas actual modern releases use `Antigravity.Manager.Tools_...` or `agm-alim_...`, and internal package versions frequently differ from the release tag. Furthermore, `fetch_api_release_tags` grepped `tag_name` without inspecting the release's `assets` array.

---

## 4. Fix Applied
1. **Pre-Flight Reachability & Asset Verification**:
   - Implemented `Test-UrlReachable` in `install.ps1` and `test_url_reachable` in `install.sh` (fast HTTP HEAD check with 4-second timeout).
   - Implemented `Get-WindowsReleaseAsset` in `install.ps1` and `parse_github_releases_py` in `install.sh` to extract the exact `browser_download_url` directly from the release metadata, completely eliminating filename template guessing.
2. **Platform Asset Filtering in All Discovery Tiers**:
   - **Tier 1 (CDN Manifest)**: Releases lacking an asset for the current OS/architecture are skipped and never added to candidate queues.
   - **Tier 2 (GitHub API)**: Each release is parsed against platform criteria (Windows `.exe`/`.zip`, Linux `.deb`/`.rpm`/`.AppImage`, macOS `.dmg`). Releases lacking platform assets are skipped.
   - **Tier 3 (`updater.json`)**: Only probed if candidates queue is empty, and requires the platform URL to pass `Test-UrlReachable` before inclusion.
3. **Pre-Download Guard**:
   Before launching download accelerators (`aria2c`), both installers check `Test-UrlReachable`. If GitHub returns 404, the installer emits an informative warning and seamlessly advances to the next candidate version without erroring out.
4. **Dynamic Stale Manifest Recovery in `install.sh`**:
   When the CDN manifest is stale relative to the installed version, `install.sh` clears stale manifest candidates, queries the live GitHub API to place valid newer releases at the head of the queue, and preserves manifest entries as fallbacks.
5. **CLI Parity**:
   Added `--dry-run` flag support to `install.sh` with safe non-destructive simulation matching `install.ps1 -DryRun`.

---

## 5. Prevention & Rules
- **Rule strictly-avoid**: NEVER guess release asset download URLs via string interpolation (e.g. `agm-alim_${ver}_x64-setup.exe` or `Antigravity.Tools_${ver}_amd64.deb`) without verifying asset existence in release metadata or performing a lightweight pre-flight HEAD reachability check.
- **Rule strictly-avoid**: NEVER populate installer candidate queues with release tags that lack binary assets for the executing platform.

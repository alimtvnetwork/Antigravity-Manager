# Plan 51: Installer Pinned Version Respect, Multi-Version Fallback Ladder & Release Ceremony

## Status: Completed
- **Created At:** 2026-09-20T21:42:30+08:00
- **Completed At:** 2026-09-20T21:45:30+08:00
- **Release:** `v4.40.0`
- **Execution Loops:** 1 continuous multi-agent loop

## Executed Objectives & Outcomes

1. **Pinned Version Detection & Prioritization in `install.ps1`:**
   - Evaluated `$PinnedVersion` (injected during release asset packaging).
   - Added regex detection on invocation environment, `$MyInvocation.Line`, and command history for URLs matching `releases/download/v?([0-9]+\.[0-9]+\.[0-9]+)/install\.ps1` (e.g. `irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.34.0/install.ps1 | iex`).
   - When pinned, binds `$candidateVersions[0]` to the requested version and queries GitHub release tag endpoint `repos/$Repo/releases/tags/v$cleanPinned` directly.
   - Built an intelligent descending fallback ladder ordering historical releases strictly `< cleanPinned` so failed pinned installs step down gracefully to earlier versions rather than jumping forward.

2. **Pinned Version Detection & Prioritization in `install.sh` & `deploy/arch/install.sh`:**
   - Evaluated `PINNED_VERSION` token and detected download URLs from command line and process execution strings.
   - Added direct querying of `repos/${REPO}/releases/tags/v${VERSION}` when pinned.
   - Built 4-candidate descending fallback ladder.

3. **Dedicated Pinned Version Section in `readme.md` & `release.yml`:**
   - Added explicit `### Pinned Version Installation (Historical Releases)` section in root `readme.md` under `## 🛠️ Install Scripts`.
   - Provided ready-to-run one-liners for PowerShell 5.1+ and Bash.
   - Verified that `.github/workflows/release.yml` formats release notes with pinned version command lines.

4. **Standardized 4-Attempt Try-Catch Fallback Ladder:**
   - Hardened `install.ps1`, `install.sh`, and `deploy/arch/install.sh` so every download and installation runs inside a try-catch loop.
   - If version 1 fails, automatically falls back to version 2, then 3, then 4.
   - If all 4 attempts fail, outputs the exact required string: `"All 4 attempts failed. I fail, so I cannot do anything."`

5. **Minor Release Ceremony (`v4.40.0`):**
   - Bumped version across all manifests to `4.40.0` via `python 03-ai-scripts/37-bump-version.py -t minor`.
   - Confirmed synchronization with `python 03-ai-scripts/14-version-sync-checker.py` (exit code 0).
   - Staged all changes in a single atomic release commit, tagged `v4.40.0`, and pushed to `origin main`.

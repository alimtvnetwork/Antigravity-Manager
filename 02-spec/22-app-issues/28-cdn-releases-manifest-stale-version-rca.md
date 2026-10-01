# 4-Part RCA 28: Stale CDN Releases Manifest (v4.85.0 Lockout) & Dynamic Manifest Generation

## Part 1: Symptoms and Blast Radius
1. **Stale Manifest Warning During Installation**:
   Whenever users ran `install.ps1` or `install.sh` on versions newer than `v4.85.0` (e.g. `v4.103.0`, `v4.109.0`), the installer emitted:
   ```text
   [*] Discovering available release versions from GitHub...
   [*] Discovered releases from CDN manifest (5 versions available, rate-limit free)
   [*] Cached CDN manifest version (v4.85.0) is not newer than current installed version (v4.103.0); querying live GitHub releases...
   ```
2. **Loss of Rate-Limit-Free Tier 1 CDN Fast-Path**:
   Because `releases-manifest.json` on `raw.githubusercontent.com` and GitHub release assets was stuck at `v4.85.0`, Tier 1 CDN resolution was skipped every time, forcing installers to fall back to Tier 2 (unauthenticated GitHub REST API), risking `403 API rate limit exceeded` for automated environments and workstations.

---

## Part 2: Proximate Cause
- `releases-manifest.json` in the git repository root was generated on 2026-09-27 with `latest_version: "4.85.0"`.
- During subsequent version bumps (`npm run bump`), `scripts/bump-version.mjs` synchronized 14 manifest files (`package.json`, `Cargo.toml`, `tauri.conf.json`, `version.json`, etc.) but completely omitted `releases-manifest.json`.
- In `.github/workflows/release.yml` line 543, the release packaging step simply copied the existing static `releases-manifest.json` into `release-files/`, uploading the stale `v4.85.0` manifest file as an asset on every newly released version.
- The original manifest generator script (`03-ai-scripts/39-generate-releases-manifest.py`) had been accidentally removed in commit `4ca4f925`.

---

## Part 3: Root Cause Analysis
1. **Absence of Atomic Bump Target for CDN Manifest**:
   The installer relies on `releases-manifest.json` for rate-limit-free version discovery. Without adding `releases-manifest.json` to the canonical `TARGET_FILES` list in `scripts/bump-version.mjs`, every version bump generated drift between repository manifests and the CDN manifest.
2. **Missing CI Release Step Generation**:
   GitHub Actions workflow `.github/workflows/release.yml` assumed `releases-manifest.json` was pre-generated and static. It never re-generated or patched the manifest with the newly released version tag, published timestamp, and platform asset links before artifact publication.
3. **Missing Local Fallback Probe**:
   Installers (`install.ps1` and `install.sh`) only probed remote URLs (`raw.githubusercontent.com` and `/releases/latest/download`), skipping local `$PSScriptRoot/releases-manifest.json` when running locally from a cloned repository.

---

## Part 4: Preventive and Remediating Actions
1. **Restored and Enhanced Manifest Generator (`03-ai-scripts/39-generate-releases-manifest.py`)**:
   - Fetches live releases from GitHub API with authentication token fallback.
   - Accepts `--target-version <ver>` to inject/prepend target releases deterministically with correct platform asset URLs (`agm-alim-setup.exe`, `agm-alim_<ver>_windows_x64.zip`, etc.).
   - Robust offline/fallback support conforming strictly to `02-spec/schema/releases-manifest.schema.json`.
2. **Integrated into Atomic Bump Workflow (`scripts/bump-version.mjs`)**:
   - Added `releases-manifest.json` to `TARGET_FILES`. Every `npm run bump` automatically updates `latest_version`, `latest_tag`, timestamps, and prepends the new release entry to `releases`.
3. **Automated CI Generation in Release Workflow (`.github/workflows/release.yml`)**:
   - Prior to packaging release files, CI now invokes:
     `python3 03-ai-scripts/39-generate-releases-manifest.py --target-version "$VER" --repo "${{ github.repository }}"`
   - Guarantees every GitHub Release asset bundle contains a fresh, 100% accurate `releases-manifest.json`.
4. **Enhanced Installer Manifest Probing (`install.ps1`, `install.sh`)**:
   - Added local directory check for `releases-manifest.json` before hitting remote network endpoints.
   - Tested and verified with `install.ps1 -DryRun`: discovers `v4.110.0` instantly with zero rate-limit fallback warnings.

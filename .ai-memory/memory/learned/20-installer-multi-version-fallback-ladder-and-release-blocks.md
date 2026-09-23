# Learned Memory: Installer Multi-Version Fallback Ladder, Release Decoupling, and Bottom-Bar Update System

## 1. Context & Motivation

During release operations and continuous user feedback for `v4.65.0`, four interconnected failure modes were identified:
1. **GitHub Release Code Block Formatting**: Combined commands with comments (`# Or pinned version:`) caused users to copy invalid syntax when using GitHub's 1-click code block copy button.
2. **Release Asset Coupling**: Packaging standalone installer scripts (`install.ps1`, `install.sh`) as release assets produced circular dependencies and caused drift between the live git repository tree and release pages.
3. **Bottom-Bar In-App Update Trigger Failure**: When users ran an installed older version (e.g., `v4.60.0`), in-app update checks failed to detect updates because the CDN-hosted `releases-manifest.json` on `main` was stale (`v4.60.0`), and notifications were only displayed if explicit flags were active on startup.
4. **Installer Fragility & Single-Failure Halts**: Installers crashed or stopped if the latest target failed, without retreating through previous releases.

## 2. Key Architectural Decisions & Solutions

### A. Release Asset Decoupling
- Standalone installer scripts (`install.ps1`, `install.sh`) reside strictly at repository root and are fetched directly from GitHub raw content endpoints (`raw.githubusercontent.com/...`).
- Updated `03-ai-scripts/29-release-orchestrator.py` to automatically execute `gh release delete-asset <tag> install.ps1 install.sh -y` post-release.
- Removed installer assets from historical and current releases (`v4.65.0`, `v4.56.0`).

### B. Multi-Version Fallback Ladder (Up to 10 Candidates)
- Implemented a 10-candidate execution queue in both `install.ps1` and `install.sh`.
- Probes CDN manifest first, then queries GitHub REST API, and falls back to embedded historical tags.
- Added stale-manifest detection comparing the manifest's top version against the installed version using SemVer. If the manifest top version is `<= current`, the installer automatically probes the GitHub API for newer releases.
- Sorts candidate queue strictly descending by SemVer so candidate index 0 is always the newest release.
- For each attempt, logs both `Release tag URL` and `Package download URL`.

### C. Clean Aria2c Delegation
- Aria2c execution runs in quiet mode (`--summary-interval=0 --console-log-level=error --show-console-readout=false`).
- Emits explicit start and completion summaries with transferred megabytes.
- Emits a clean delegation notice on error before handing off to `curl` or `Invoke-WebRequest`.

### D. Dedicated Release Page Markdown Code Blocks
- Formatted both `readme.md` and release bodies with 4 isolated Markdown code blocks:
  - PowerShell Direct Latest
  - PowerShell Pinned Version
  - Bash Direct Latest
  - Bash Pinned Version
- Enables seamless 1-click copying without manual text selection.

### E. Frontend Centralized Update Store & Bottom-Bar Indicator
- Created `src/stores/use-update-store.ts` to manage update status reactively.
- Added `centerContent` slot to `Pagination.tsx` desktop view.
- Added interactive update pill (`Sparkles` + `Update Available: v<version>` + `Install Now`) in `Accounts.tsx` bottom bar, giving users immediate visibility and 1-click update installation.

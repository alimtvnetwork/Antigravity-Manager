# Antigravity Tools v4.264.0

### Quick Install

**Windows (PowerShell):**
```powershell
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.264.0/install.ps1 | iex
```

**macOS / Linux (Bash):**
```bash
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.264.0/install.sh | bash
```

### Highlights & Fixes

- **Release Gate In-Progress CI Polling**: Updated `.github/workflows/release.yml` with a bounded polling loop (polling every 20s up to 25 minutes) to await in-progress or queued `ci.yml` runs triggered on tag push, eliminating premature gate failure (`conclusion: missing`) and release refusal (Thanks to @aukgit)
- **Root Cause Analysis (RCA 47)**: Authored comprehensive 4-part RCA in `.ai-memory/cicd-issues/47-release-gate-ci-race-condition-rca.md` documenting the GitHub Actions event race condition and indexing recurring CI race conditions in `.ai-memory/cicd-issues/readme.md` (Thanks to @aukgit)

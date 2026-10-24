# Antigravity Tools v4.265.0

### Quick Install

**Windows (PowerShell):**
```powershell
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.265.0/install.ps1 | iex
```

**macOS / Linux (Bash):**
```bash
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.265.0/install.sh | bash
```

### Highlights & Fixes

- **CI Concurrency SHA Scoping**: Scoped CI concurrency group on `main` in `.github/workflows/ci.yml` to the commit SHA (`group: ${{ github.workflow }}-${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}`), ensuring sequential commits on `main` (such as release notes pushes or documentation updates) never cancel in-flight CI workflows (Thanks to @aukgit)
- **Root Cause Analysis (RCA 47 Expansion)**: Documented GitHub Actions concurrency group collision and cancel-in-progress dynamics in `.ai-memory/cicd-issues/47-release-gate-ci-race-condition-rca.md` (Thanks to @aukgit)
- **Release Gate Resilience Verification**: Verified bounded polling loop in `.github/workflows/release.yml` across active releases, ensuring multi-platform desktop binaries and installers build and publish cleanly (Thanks to @aukgit)

# Subtask: CI/CD Bottleneck Analysis & Local Dev Runner (run.ps1)

**Target Files:**
- `run.ps1`
- `02-spec/12-cicd-pipeline-workflows/15-release-bottleneck-analysis.md`
- `.ai-memory/issues/46-cicd-release-bottlenecks.md`

**Action:**
1. In `run.ps1`:
   - Create a clean, fast PowerShell runner script at repository root supporting:
     - `.\run.ps1 -Dev` (starts `npm run dev` and `npm run tauri dev`)
     - `.\run.ps1 -Build` (local debug build check)
     - `.\run.ps1 -Check` (fast local TypeScript + cargo fmt verification)
2. In `02-spec/12-cicd-pipeline-workflows/15-release-bottleneck-analysis.md`:
   - Analyze GitHub Actions workflows (`.github/workflows/ci.yml` and `release.yml`).
   - Identify top bottlenecks:
     - APT package installation on Ubuntu runners (heavy WebKit2GTK and dependencies taking 3–6 minutes per job).
     - Triple platform matrix (Ubuntu, Windows, macOS) executed redundantly in both CI and Release workflows.
     - Unsigned Tauri debug build compiling redundant debug artifacts before release builds.
     - Rust cargo cache hit rate vs cache key invalidation on Cargo.lock updates.
   - Outline concrete recommendations for pipeline speedup (e.g. pre-installed Docker images for Linux, workflow concurrency filters, conditional job triggers).

**Constraints:**
- Cross-platform PowerShell safety.
- Relative git paths only.

# Issue 46: CI/CD & Release Pipeline Bottlenecks Analysis

## 1. Context
Analysis of GitHub Actions workflows (`.github/workflows/ci.yml` and `release.yml`) revealed high run times and excessive consumption of runner minutes.

## 2. Identified Bottlenecks
- **APT Package Re-download:** Both `check-rust` and `build-tauri` download ~200MB of WebKit2GTK dependencies on Ubuntu runners, taking 3–6 minutes per job.
- **Excessive Matrix in CI:** Running full Tauri debug builds across Ubuntu, Windows, and macOS on every push burns 7 jobs and 45+ billable minutes per commit.
- **Redundant Debug Build:** `npm run tauri build -- --debug --no-bundle` duplicates `cargo check` and `cargo test`.
- **macOS Billing Multiplier:** macOS runners cost 10x standard Linux rates, making redundant CI matrix execution expensive.

## 3. Implemented Solutions & Next Steps
- Created local runner `run.ps1` at repository root supporting `.\run.ps1 -Check` (runs in ~8 seconds locally to prevent failed remote CI runs).
- Documented complete architectural recommendations in `02-spec/12-cicd-pipeline-workflows/23-release-bottleneck-analysis.md`.
- Proposed containerized builder for Ubuntu and path-filtered triggers for PRs.

# Subtask 08 — Verification, Bump & CI/CD Pipeline

## Status: Pending

## Scope
1. Pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. Version bump & release notes:
   - Run `npm run bump minor`
   - Synchronize changelogs (`CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md`) with `@aukgit` attribution.
3. Commit and CI/CD verification:
   - Commit via GitMap: `gitmap cpb "instance-and-ui - fix instance cloning settings projects audit dark theme and about ui"`
   - Check CI/CD status with GitMap (`gitmap pipeline-ai status`).

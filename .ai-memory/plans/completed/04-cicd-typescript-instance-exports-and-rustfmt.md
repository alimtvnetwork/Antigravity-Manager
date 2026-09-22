# CI/CD Task: TypeScript Instance Exports and Rustfmt Drift

## Status: Completed

## Source
- Runner jobs: TypeScript Check & Rust Format Check
- Error type: FAIL
- Detected at: 2026-09-22T19:55:00+08:00

## Error Summary
```text
src/stores/useInstanceStore.ts(165,35): error TS2339: Property 'wipeSession' does not exist on type 'typeof import(".../src/services/instanceService")'.
src/stores/useInstanceStore.ts(304,45): error TS2339: Property 'pickBestCandidateAccount' does not exist on type 'typeof import(".../src/services/instanceService")'.
src/stores/useInstanceStore.ts(346,46): error TS2339: Property 'selectNextBestProfile' does not exist on type 'typeof import(".../src/services/instanceService")'.
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check failed on build.rs, auto_switcher.rs, process.rs, repo_db.rs
```

## Required Fix
1. Export `wipeSession`, `pickBestCandidateAccount`, and `selectNextBestProfile` from `src/services/instanceService.ts`.
2. Format Rust codebase via `cargo fmt --manifest-path src-tauri/Cargo.toml`.

## Acceptance Criteria
- [x] `node node_modules/typescript/bin/tsc --noEmit` reports 0 errors.
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes with exit code 0.
- [x] Quality gates in `03-ai-scripts/06-cicd-local-runner.py` pass (36/36 gates passed).

## Resolution Details
- Added `export const wipeSession = wipeInstanceSession;` and `export const selectNextBestProfile = findBestRotationProfile;` aliases to `src/services/instanceService.ts`.
- Implemented and exported `pickBestCandidateAccount` in `src/services/instanceService.ts` utilizing `findSmartRotationAccount` and `findBestSmartPlayAccount`.
- Formatted Rust codebase with `cargo fmt --manifest-path src-tauri/Cargo.toml`.
- Ran `python 03-ai-scripts/31-md-gap-fixer.py --fix` and verified 36/36 quality gates pass green.

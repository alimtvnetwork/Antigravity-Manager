# CI/CD Issue: TypeScript Instance Service Missing Exports and Rustfmt Drift

- Job: TypeScript Check / Rust Format Check
- Type: FAIL
- Detected: 2026-09-22T19:55:00+08:00
- Status: resolved

## Error
```text
src/stores/useInstanceStore.ts(165,35): error TS2339: Property 'wipeSession' does not exist on type 'typeof import(".../src/services/instanceService")'.
src/stores/useInstanceStore.ts(304,45): error TS2339: Property 'pickBestCandidateAccount' does not exist on type 'typeof import(".../src/services/instanceService")'.
src/stores/useInstanceStore.ts(346,46): error TS2339: Property 'selectNextBestProfile' does not exist on type 'typeof import(".../src/services/instanceService")'.
cargo fmt check failed on src-tauri build.rs, auto_switcher.rs, process.rs, repo_db.rs
```

## Root Cause
- `useInstanceStore.ts` consumed helper functions (`wipeSession`, `pickBestCandidateAccount`, `selectNextBestProfile`) that were renamed or not directly re-exported in `src/services/instanceService.ts`.
- In addition, recent Rust backend changes across `build.rs`, `auto_switcher.rs`, `process.rs`, and `repo_db.rs` had minor indentation and trailing newline differences that triggered `cargo fmt --check` failure.

## Fix Applied
- Export `wipeSession = wipeInstanceSession`, `selectNextBestProfile = findBestRotationProfile`, and implement `pickBestCandidateAccount` in `src/services/instanceService.ts`.
- Run `cargo fmt --manifest-path src-tauri/Cargo.toml` to format Rust files.

## Plan Task
Enqueued at `.ai-memory/plans/pending/13-cicd-typescript-instance-exports-and-rustfmt.md`

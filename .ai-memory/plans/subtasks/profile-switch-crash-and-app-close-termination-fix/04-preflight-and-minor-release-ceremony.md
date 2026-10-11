# Subtask 04: Preflight & Minor Release Ceremony

## Context
Execute minor version bump from `v4.186.0` to `v4.187.0`, synchronize all 14 project locations, update documentation with strict `@aukgit` attribution, run pre-flight gates, and release.

## Files to Modify
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `changelog.md`
- `changelog_en.md`
- `README.md`
- `README_EN.md`

## Acceptance Criteria
1. `npm run bump minor` synchronizes versions cleanly.
2. Attribution strictly credits `@aukgit` (`(Thanks to @aukgit)`).
3. Pre-flight checks (`cargo fmt -- --check`) pass with 0 errors.
4. GitMap atomic commit (`gitmap cpf`), git tag `v4.187.0`, and push to origin.

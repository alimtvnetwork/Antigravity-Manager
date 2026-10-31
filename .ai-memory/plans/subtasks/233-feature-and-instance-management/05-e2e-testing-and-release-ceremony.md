# Subtask 05: E2E Quality Gates, Minor Version Bump & Release Ceremony

## Target Files
- `package.json`
- `src-tauri/Cargo.toml`
- `changelog.md`
- `changelog_en.md`
- `README.md`
- `README_EN.md`

## Objectives
1. Run pre-flight gates:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. Run targeted test suites and verify 0 errors in `errors.db` (`agm errors count`).
3. Execute atomic minor version bump: `npm run bump minor` (`v4.274.0`).
4. Update `changelog.md` and `changelog_en.md` attributing strictly `@aukgit` (`(Thanks to @aukgit)`).
5. Synchronize `README.md` and `README_EN.md` release notes.
6. Commit, tag (`v4.274.0`), and push to `origin/main` + tag.

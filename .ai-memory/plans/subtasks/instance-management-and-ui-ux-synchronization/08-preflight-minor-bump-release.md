# Subtask 08: Preflight Checks, Minor Version Bump & Release Ceremony

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `08-preflight-minor-bump-release`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Execute preflight checks (`cargo fmt`, `cargo clippy`, `npm run build`), bump minor version via `npm run bump minor`, synchronize `README.md`, `README_EN.md`, `CHANGELOG.md`, `changelog_en.md`, commit using GitMap hyphen-separated format, and push to upstream Git.

## Scope of Changes
- Run `cargo fmt -- --check` in `src-tauri`.
- Run `cargo clippy --all-targets --all-features` in `src-tauri`.
- Run `npm run build`.
- Run `npm run bump minor`.
- Audit attribution in changelog files to strictly `@aukgit`.
- Ensure strictly relative paths in changelog and release notes.
- Synchronize root `README.md` and `README_EN.md` update logs.
- Git commit and push via GitMap.

## Verification
- All preflight checks succeed with 0 errors.
- Version bumped consistently across 14 locations.
- Git working tree clean after push.

# Subtask 06: End-to-End Verification, Pre-flight Checks, and Minor Bump Release Ceremony

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 06`

## Goal
Verify all fixes locally, run pre-flight checks (`cargo fmt`, tests, `tsc`, `npm run build`), bump minor version (`v4.282.0`), synchronize changelogs with attribution strictly `@aukgit`, commit, tag, and push.

## Steps
1. Execute `cd src-tauri && cargo fmt -- --check`.
2. Run frontend unit tests and type checks.
3. Build frontend bundle (`npm run build`).
4. Bump version: `npm run bump minor`.
5. Audit changelogs to ensure attribution is strictly `@aukgit` (`(Thanks to @aukgit)`).
6. Commit, tag `v4.282.0`, and push to remote.

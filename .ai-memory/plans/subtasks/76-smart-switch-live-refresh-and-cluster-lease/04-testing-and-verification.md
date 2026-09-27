# Subtask 04: Quality Gates, Unit Tests & Version Release

- [x] Write targeted unit tests for `proxy_disabled` candidate exclusion and 100% quota gate.
- [x] Run pre-flight checks:
  - [x] `cd src-tauri && cargo fmt -- --check`
  - [x] `cd src-tauri && cargo clippy --lib --bin agm`
  - [x] `npm run build`
- [x] Bump version: `node scripts/bump-version.mjs minor` (bump to `4.82.0`).
- [x] Update `CHANGELOG.md` and `CHANGELOG_EN.md`.
- [x] Move plan to `completed/` and execute atomic git commit and push.

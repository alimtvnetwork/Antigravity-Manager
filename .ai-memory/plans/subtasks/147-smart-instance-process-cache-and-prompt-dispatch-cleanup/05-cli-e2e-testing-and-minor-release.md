# Subtask 05: CLI Verification, E2E Testing & Minor Release Ceremony

## Objective
Perform end-to-end testing, verify CLI parity for prompt dispatch and process counting, run comprehensive pre-flight gates (`cargo fmt`, `cargo clippy`, `npm run build`), and execute minor release ceremony to `v4.165.0`.

## Actions
1. Verify CLI commands:
   - `agm doctor`
   - `agm prompts ls`
   - `agm prompts tree`
2. Run pre-flight verification:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
3. Execute Minor Release Ceremony:
   - Run `npm run bump minor` (updating all package manifests to `4.165.0`).
   - Audit git log and update `CHANGELOG.md` and `CHANGELOG_EN.md` with attribution strictly to `@aukgit` (`(Thanks to @aukgit)`).
   - Synchronize release summary into `README.md` (under "## 📝 更新日志") and `README_EN.md` (under "## 📝 Changelog").
   - Commit atomic release changeset, create tag `v4.165.0`, and push to upstream `main`.

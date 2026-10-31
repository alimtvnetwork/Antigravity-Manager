# Subtask 07: Minor Version Bump & Release Ceremony

## Objectives
1. Run minor version bump via `npm run bump minor`.
2. Update `changelog.md` and `changelog_en.md` with release notes adhering to the `@aukgit` attribution rule and strict relative paths.
3. Synchronize `README.md` and `README_EN.md` changelog sections.
4. Execute final preflight checks (`cargo clippy`, `cargo fmt`, `npm run build`).
5. Stage, format, and push atomic GitMap commit (`gitmap cpf "instances - implement unified lifecycle management, caller preservation and ui/ux synchronization"`).

## Owned Files
- `package.json`
- `src-tauri/Cargo.toml`
- `changelog.md`
- `changelog_en.md`
- `README.md`
- `README_EN.md`

## Verification
- Working tree clean after GitMap push.

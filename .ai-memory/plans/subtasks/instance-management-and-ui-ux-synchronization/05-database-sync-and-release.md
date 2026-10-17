# Subtask 05: Database State Synchronization & Release Ceremony

Owner: Lead
Target Files:
- `.ai-memory/plans/readme.md`
- `02-spec/21-app/readme.md`
- `package.json`, `Cargo.toml`, `version.json`
- `changelog.md`, `changelog_en.md`
- `readme.md`, `readme_en.md`

## Requirements
1. Verify that SQLite database files (`task_index.db`, `history-*.db`, `fleet_sync.db`, `repo_prompts.db`) maintain valid transactional integrity and updated records with zero orphaned entries during test instance creation, switching, and removal.
2. Run pre-flight checks:
   - `cargo fmt -- --check`
   - `cargo clippy --all-targets --all-features`
   - `npm run build`
3. Execute minor version bump via `npm run bump minor`.
4. Synchronize changelogs with attribution strictly `(Thanks to @aukgit)` and relative paths only.
5. Update `readme.md` and `readme_en.md` with new release summary.
## Status: COMPLETED
- All pre-flight checks verified (fmt, clippy, frontend build, automated e2e).
- Bumped to v4.217.0 across 13 manifest locations.
- Synchronized changelog.md, changelog_en.md, readme.md, and readme_en.md with strict @aukgit attribution.
- Database integrity verified with no orphaned test entries.

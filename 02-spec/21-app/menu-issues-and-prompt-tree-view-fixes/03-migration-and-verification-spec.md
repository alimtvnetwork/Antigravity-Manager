# Migration & Verification Specification: Quality Gates, Attributions, & Release

## 1. Pre-flight Quality Gates
Before any commits or release tagging, the following gates must execute and pass with zero warnings/errors:
1. `cd src-tauri && cargo fmt -- --check`
2. `cd src-tauri && cargo clippy --all-targets --all-features`
3. `npm run build`

## 2. Release & Versioning Invariants
1. **Atomic Version Bump**:
   - Run `npm run bump patch` to increment version in all 14 project manifest files.
2. **Attribution Invariant**:
   - Attribution in `changelog.md` and `changelog_en.md` must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).
   - No other user handles may appear in changelogs.
3. **Synchronize READMEs**:
   - For stable releases on `main`, update both `readme.md` and `readme_en.md` with the release notes.
4. **Strictly Relative Git Paths**:
   - Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`) in specs, plans, commits, changelogs, or release notes.
5. **Caller IDE Self-Preservation**:
   - Never terminate active caller instance `default-copy-5597`.

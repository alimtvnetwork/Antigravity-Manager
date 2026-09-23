# Antigravity Tools Release SOP (Standard Operating Procedure)

Detailed release guidelines. Merge gates and release boundaries follow the root `agents.md`.

---

## 1. Flow Overview

```text
[Channel A: Production Release]
Pre-flight ─► checkout main ─► npm run bump <patch|minor> ─► Changelog ─► git push origin main ─► Tag vX.Y.Z ─► Auto-publish Latest Release

[Channel B: Beta Pre-release]
Pre-flight ─► checkout beta ─► npm run bump beta ──────────► Changelog ─► git push origin beta ─► Tag vX.Y.Z-beta.N ─► Auto-publish Pre-release (Isolated)
```

> **Channel Isolation & Maintainer Collaboration Principles**:
> - **Production Channel (Main)**: `main` is an **unconditionally clean release branch**, releasing only pure SemVer versions (e.g. `v4.7.14`). The pipeline strictly blocks any pre-release tags containing `-`.
> - **Preview Channel (Beta)**: `beta` is an **independent pre-release branch**. All pre-release testing versions (e.g. `v4.7.14-beta.1`, `-cleaned`, etc.) must be committed here and triggered by `beta` for isolated builds. Pre-release artifacts are automatically marked as Pre-release and never tagged Latest, having zero impact on production users.
> - **Staging on Beta First**: For new features, major refactors, or high-risk fixes, **maintainers must be consulted** before deciding whether to stage and verify on the `beta` branch first. Once verified stable (or confirmed through beta preview testing), changes may be merged into `main`.

---

## 2. Release Steps

### Step 0: Pre-flight Checks

Ensure the workspace is clean and run the exact same verification commands as CI:

```bash
git checkout main && git pull origin main
git status          # Must show: nothing to commit, working tree clean

cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features
cargo check
cd ..
npm run build
```

> CI quality gates fully cover both `main` and `beta` branches. Ensure pre-flight checks pass on `main` before a production release, and on `beta` before a beta release.

### Step 1: Atomic Version Synchronization

`scripts/bump-version.mjs` synchronizes repository-wide version numbers and generates a CHANGELOG skeleton in one step:

| Scenario | Command | Example |
| --- | --- | --- |
| Patch (Bugfix / Performance) | `npm run bump patch` | 4.7.13 → 4.7.14 |
| Minor (New Features) | `npm run bump minor` | 4.7.13 → 4.8.0 |
| Major (Breaking Changes) | `npm run bump major` | 4.7.13 → 5.0.0 |
| Beta Pre-release Increment | `npm run bump beta` | 4.7.13 → 4.7.14-beta.1 (beta.1 → beta.2) |
| Derivative / Custom Pre-release | `npm run bump 4.7.14-cleaned` | Dual release (`-beta` / `-cleaned` / `-rc`) |
| Any Valid SemVer | `npm run bump 4.8.0` | — |

**Optional Flags**: `--dry-run` performs a dry run without modifying disk; `--commit` automatically creates a `chore(release): bump version to ...` commit.

**Synchronized Files**: `package.json`, `package-lock.json` (root version mirror, two locations), `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `Casks/antigravity-tools.rb`, `README.md`, `README_EN.md`, `src/components/layout/MiniView.tsx`, `src/pages/Settings.tsx`, `CHANGELOG.md`, `CHANGELOG_EN.md`.

> Version matching uses **structural anchoring** instead of exact version string matching: `package-lock.json` anchors by field position; README anchors by `(v<digits>...)` / `Version-<digits>...-blue`. This ensures subsequent syncs correctly match even if earlier pre-release rounds skipped README updates.

**Guard Rails**: Target version must be strictly higher than current version; regressions are prevented by hard aborts.

**Pre-release Version Format**: `npm run bump beta` generates `X.Y.Z-beta.N` (`beta.1` for the first run, incrementing to `beta.2` subsequently). This version string defines the CHANGELOG skeleton heading and the subsequent Tag name; **all three must match exactly**.

### Step 2: Trace Commits & Update Changelogs

Before writing changelog entries, **perform a complete retrospective based on Git commit history and merged PRs** to avoid missing contributor credits or issue references:

```bash
# 1. Scan all commits, Author, and Co-Authored-By trailers since the previous tag
git log $(git describe --tags --abbrev=0)..HEAD --format="Commit: %h | %an <%ae> | %s%n%(trailers:key=Co-Authored-By)"

# 2. List merged PRs and linked issues during this period
gh pr list --state merged --limit 20
```

Based on the inventory, populate the release skeleton:

- **Mandatory Issue / PR Linking**: Entry titles must include the corresponding issue/PR number (e.g. `(PR #3504)` or `(Fixes #3499, #3501)`);
- **Mandatory Contributor Credits**: All external contributors identified from git commits or PRs must be explicitly credited using `(Thanks to @username)`. The Release page **Contributors avatar list is automatically extracted from these tags**;
- **Format Example**:
```markdown
*   **Version History**:
    *   **v4.7.14 (2026-09-23)**:
        -   **[Core] Architecture Refactor and Optimization (PR #3504)**:
            -   **Details**: Description of implementation details.
        -   **[Bugfix] External Contributor Fix (Fixes #3508, Thanks to @username)**:
            -   **Details**: Acknowledgement and fix details.
```

> 1. **Heading must match Tag character-for-character**: The release pipeline uses `awk` to match the changelog heading using the tag name (`github.ref_name`, with `v` prefix). The `v` prefix and full pre-release suffix must match exactly. For example, `npm run bump beta` produces `X.Y.Z-beta.1`, so the tag must be `vX.Y.Z-beta.1` and the heading must be `**vX.Y.Z-beta.1 (YYYY-MM-DD)**`. If mismatched, release body text will silently fall back to `See the assets to download this version and install.`.
> 2. With `generateReleaseNotes: true` enabled, GitHub automatically appends `What's Changed` and `New Contributors` (including PR links and contributor profiles).
> 3. **Beta Releases do not enter README**: Pre-release tags containing `-` (`-beta`, `-cleaned`, `-rc`, etc.) are **recorded only in `CHANGELOG.md`** and must not be written to README version badges or latest version sections. README always reflects the latest **stable** release. `bump-version.mjs` automatically skips README updates for pre-releases.
> 4. **Inline Contributor Credits**: Credit external contributors inline as `(Thanks to @username)`. The Release page avatar list is automatically populated from `@username` mentions.

### Step 3: Commit and Push Target Branch

```bash
# Production Release: commit and push to main branch
git checkout main
git add -A
git commit -m "chore(release): bump version to 4.7.14 and update changelog"
git push origin main

# Beta Pre-release: commit and push to beta branch (never push to main)
git checkout beta
git add -A
git commit -m "chore(release): bump version to 4.7.14-beta.1 and update changelog"
git push origin beta
```

### Step 4: Create and Push Tag

```bash
# Production Release: create numeric tag from main (triggers release, updates Latest)
git tag v4.7.14
git push origin v4.7.14

# Beta Pre-release: create pre-release tag from beta (triggers isolated build, does not update Latest)
git tag v4.7.14-beta.1
git push origin v4.7.14-beta.1
```

> Tag string must match the CHANGELOG heading character-for-character (`v` prefix + full pre-release suffix).

**Strict Branch & Tag Gates**:
- Pre-release tags containing `-` created on `main`-exclusive commits will be blocked immediately by CI, rejecting the build.
- Stable tags without `-` created on `beta`-exclusive commits will be blocked immediately by CI.
- Pre-release builds automatically downgrade to NSIS, never update Latest, and never update stable users' `updater.json`.

### Step 5: Verification

Once the tag is pushed, the automated pipeline takes over:

1. **Progress**: Check the `Release` workflow on GitHub `Actions`.
2. **Build Matrix**: Windows (`.msi` / NSIS `.exe`), macOS (`.dmg`, Apple Silicon and Intel dual architectures), Linux (`.AppImage` / `.deb` / `.rpm`), Docker multi-arch images pushed to Docker Hub; outputs `updater.json` (signed when `TAURI_SIGNING_PRIVATE_KEY` is configured).
3. **Verification**: In ~10-15 minutes, verify `Antigravity Tools vX.Y.Z` and attached artifacts on GitHub `Releases`.

---

## 3. Troubleshooting & Recovery

### 1. Delete Accidental Tag

```bash
git tag -d v4.7.14
git push origin :refs/tags/v4.7.14
```

### 2. Version Guard Rail Abort

The error `✗ Error: Guard rail triggered: Target version [...] must be strictly greater than current version [...]!` indicates the target version is less than or equal to current. Versions must monotonically increase. Pass a higher version (e.g. `npm run bump patch`).

# Antigravity Tools Release Operation Guide (Release SOP)

Detailed release procedures. The red line for submission access control and release is based on the root directory `AGENTS.md`.

---

## 1. Process Overview

```text
[Channel A: Official version released]
Pre-flight ─► checkout main ─► npm run bump <patch|minor> ─► Supplementary log ─► git push origin main ─► Tag vX.Y.Z ─► Automatically release Latest Release

[Channel B: Beta Pre-release]
Pre-flight ─► checkout beta ─► npm run bump beta ──────────► Supplementary log ─► git push origin beta ─► Tag vX.Y.Z-beta.N ─► Automatically release Pre-release (no sense of isolation)
```

> **Principles of channel isolation and maintainer collaboration**:
> - **Official version channel (Main)**: `main` is the **absolutely pure official versioning branch**. Only purely digital official versions (such as `v4.7.14`) are released. The pipeline strictly intercepts any pre-release tags with `-`.
> - **Preview channel (Beta)**: `beta` is the **independent pre-release versioning branch**. All pre-release test versions (such as `v4.7.14-beta.1`, `-cleaned`, etc.) are submitted here and are triggered by `beta` to independently build. Pre-release products are automatically marked as Pre-release and never hit Latest, which does not affect official moderator user updates at all.
> - **Staging on Beta First**: If new features, major refactorings or high-risk fixes are involved, **must proactively ask the maintainer** whether to modify and verify on the `beta` branch first. It can be merged into `main` only after it is verified to be stable (or the beta preview version is released for internal testing).

---

## 2. Operation steps

### Step 0: Pre-flight (Pre-flight)

Make sure the workspace is clean and execute the exact same preflight command as CI on the commits that will be flagged:

```bash
git checkout main && git pull origin main
git status # should show nothing to commit, working tree clean

cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features
cd ..
npm run build
```

> CI access control has fully covered the `main` and `beta` branches. Make sure to pass the `main` pre-flight before official release, and make sure to pass the `beta` pre-flight before beta pre-release.

### Step 1: Atomic synchronization of version numbers

`scripts/bump-version.mjs` synchronizes the version number of the entire warehouse with one click and generates a CHANGELOG skeleton:

| Scenario | Command | Example |
| --- | --- | --- |
| Patches (Bugfix/Performance) | `npm run bump patch` | 4.7.13 → 4.7.14 |
| Minor version (new features) | `npm run bump minor` | 4.7.13 → 4.8.0 |
| Major version (breaking changes) | `npm run bump major` | 4.7.13 → 5.0.0 |
| pre-release bump | `npm run bump beta` | 4.7.13 → 4.7.14-beta.1 (beta.1 → beta.2) |
| Specify the derived version | `npm run bump 4.7.14-cleaned` | Dual versions with the same baseline (`-beta` / `-cleaned` / `-rc`) |
| Specify any valid SemVer | `npm run bump 4.8.0` | — |

**Optional parameters**: `--dry-run` only drills and does not write to disk; `--commit` automatically generates `chore(release): bump version to...` ​​submission.

**Synchronization scope**: `package.json`, `package-lock.json` (root version mirror, two places), `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `Casks/anti gravity-tools.rb`, `README.md`, `README_EN.md`, `src/components/layout/MiniView.tsx`, `src/pages/Settings.tsx`, `CHANGELOG.md`, `CHANGELOG_EN.md`.

> Version string matching uses **structure anchoring** instead of exact matching of the current version number: `package-lock.json` is anchored by field position, and README is anchored by `(v number...)` / `Version-number...-blue` form. In this way, even if the version string lags due to historical rounds (such as skipping README in pre-release), the next round of synchronization can still hit the target correctly.

**Built-in foolproof**: The target version must be strictly higher than the current version, otherwise it will be intercepted in red to prevent version rollback.

**Pre-release version number form**: `npm run bump beta` generates `X.Y.Z-beta.N` (the first time is `beta.1`, and subsequent increments are `beta.2`). This version string also determines the CHANGELOG skeleton title and subsequent Tag name inserted by the script. The three must be completely consistent (see the tips in step 2 and step 4 for details).

### Step 2: Backtrack Commit and Supplementary Update Log

Before filling in the change log, **a complete traceback must be conducted based on the objective facts of Git submission history and merged PR** to prevent missing contributor signatures or issue associations:

```bash
# 1. Scan all submissions, Author and Co-Authored-By signatures since the previous version
git log $(git describe --tags --abbrev=0)..HEAD --format="Commit: %h | %an <%ae> | %s%n%(trailers:key=Co-Authored-By)"

# 2. List the PRs and associated issues merged during this period
gh pr list --state merged --limit 20
```

Based on the inventory results, fill in the core highlights in the version skeleton inserted by the script:

- **Mandatory association Issue / PR**: The entry title must contain the corresponding source number (such as `(PR #3504)` or `(Fixes #3499, #3501)`);
- **Force in-line acknowledgments to be unified**: Acknowledgments are unified and strictly marked as `(Thanks to @aukgit)`, and any other GitHub user handle (`@...`) is strictly prohibited. Ensure that the Contributors list on the Release page strictly only displays aukgit;
- **Format Example**:
```markdown
* **Version evolution**:
    *   **v4.7.14 (2026-09-23)**:
        - **[Core Category] Functional Refactoring and Optimization (PR #3504)**:
            - **Function Details**: Core implementation description.
        - **[Core Category] Core updates and optimizations (Fixes #3508, Thanks to @aukgit)**:
            - **Function Details**: Core implementation description.
```

> 1. **The title must be consistent character by character with the Tag**: The pipeline uses `awk` to match the CHANGELOG title line with the tag name (`github.ref_name`, including the `v` prefix), and the **`v` prefix and the complete pre-release suffix must be word-for-word**. The version number added by `npm run bump beta` is in the shape of `X.Y.Z-beta.1`, so the Tag should be `vX.Y.Z-beta.1` (not `vX.Y.Z-beta`), and the title must also be written as `**vX.Y.Z-beta.1 (date)**`. If there is no match, the main text will silently degenerate into the placeholder copy `See the assets to download this version and install.`.
> 2. `generateReleaseNotes: false` has been set to prevent GitHub from automatically extracting the submission history to generate a non-aukgit Contributors list.
> 3. **Beta versions do not enter the README**: Tags containing pre-release/derivative versions with `-` (`-beta` / `-cleaned` / `-rc`, etc.) **are only recorded in `changelog.md`**, and no version number, Shields badge or "latest version" paragraph of any README is allowed. README always reflects only the latest **official version**. `bump-version.mjs` has this judgment built in: the pre-release version automatically skips the two READMEs and only synchronizes the remaining version configuration files.
> 4. **Contributor acknowledgments are unified**: Acknowledgments are unified with `(Thanks to @aukgit)`. The Contributors avatar list on the Release page strictly only displays aukgit.
> 5. **The official version release is forced to synchronize the bilingual README update log**: When releasing the official version, in addition to updating `changelog.md` (and `changelog_en.md`), the important update summary of the latest official version must also be synchronized to `readme.md` (`## 📝 Update log`) and `readme_en.md` (`## 📝 Changelog`). Only updating the changelog is strictly prohibited. The latest version instructions on the README homepage are missing.

### Step 3: Commit and push the target branch

```bash
# Official version: Submit and push to main branch
git checkout main
git add -A
git commit -m "chore(release): bump version to 4.7.14 and update changelog"
git push origin main

# Beta pre-release version: submit and push to beta branch (it is strictly prohibited to push to main, keep main pure)
git checkout beta
git add -A
git commit -m "chore(release): bump version to 4.7.14-beta.1 and update changelog"
git push origin beta
```

### Step 4: Tag and push

```bash
# Official version: Create a pure digital tag from main (trigger official release, update Latest)
git tag v4.7.14
git push origin v4.7.14

# Beta pre-release version: Tag pre-release from beta (triggers isolation build, does not update Latest)
git tag v4.7.14-beta.1
git push origin v4.7.14-beta.1
```

> The Tag string must be **character-for-character identical** to the title of this release in CHANGELOG (`v` prefix + full pre-release suffix), otherwise the Release body will degenerate into placeholder copy.

**Strict access control for branches and labels**: The pipeline has built-in branch and label matching assertions.
- If the pre-release tag with `-` is placed on the `main` unique submission, the pipeline will immediately block it and refuse to build and release;
- If the official version of Tag without `-` is placed on a `beta` unique submission, the pipeline will immediately intercept it.
- The pre-release version is automatically downgraded to NSIS, Latest is not updated, and the `updater.json` of official users is not updated. The main user client will not be affected in any way.

### Step 5: Acceptance

After pushing the Tag, the pipeline automatically takes over without manual intervention:

1. **Progress**: `Release` workflow of the warehouse `Actions` page;
2. **Build matrix**: Windows (`.msi` / NSIS `.exe`), macOS (`.dmg`, Apple Silicon and Intel dual architecture), Linux (`.AppImage` / `.deb` / `.rpm`), Docker multi-architecture image push to Docker Hub; output `updater.json` (configuration `TAURI_SIGNING_PRIVATE_KEY` including signature);
3. **Acceptance**: After about 10~15 minutes, confirm that `Antigravity Tools vX.Y.Z` and accessories are complete on the `Releases` page.

---

## 3. Abnormalities and emergencies

### 1. Delete accidentally typed Tags

```bash
git tag -d v4.7.14
git push origin :refs/tags/v4.7.14
```

### 2. Fool-proof protection error reporting

Error `✗ Error: Foolproof protection is in effect: the target version number [...] must be strictly higher than the current version number [...]! ` Indicates that the target version is less than or equal to the current version. The version number must strictly increase monotonically, please pass in a higher version (such as `npm run bump patch`).

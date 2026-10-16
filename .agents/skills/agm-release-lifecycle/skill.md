---
name: agm-release-lifecycle
description: Specialized skill for managing the Antigravity-Manager atomic 14-location version bump pipeline, dual-channel release gates (main vs beta), strict @aukgit attribution invariant, pre-flight checks, thinking store invalidation toggles, and multi-architecture packaging.
---

# AGM Release Lifecycle & Version Ceremony Architecture

This skill provides comprehensive architectural guidance, invariants, and operational procedures for executing release ceremonies, atomic version synchronization, documentation updates, and multi-platform distribution in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

Antigravity-Manager enforces a strict, atomic multi-manifest release discipline governed by `scripts/bump-version.mjs`, GitHub Actions release gates, and strict repository governance:

```
+----------------------------------------------------------------------------------------------------+
|                                      Release Intent & Branch Isolation                             |
|      Stable Releases (正式版) -> main branch          Preview Releases (Beta) -> beta branch       |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                               Atomic Multi-Manifest Version Sync                                   |
|                                    npm run bump <type|version>                                     |
|  - 14 distinct files synchronized simultaneously via scripts/bump-version.mjs                      |
|  - Automatic changelog skeleton generation with strict @aukgit attribution                         |
|  - Stable: Synchronizes README.md & README_EN.md release summaries                                 |
|  - Beta: Keeps README intact, updates only changelogs and code manifests                           |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                             Thinking Store Invalidation Toggle Check                               |
|        src/components/common/SuggestionDeleteThinkingModal.tsx                                     |
|  - Routine release: SUGGESTION_DELETE_THINKING_STORE = false                                       |
|  - Architecture/Schema refactor: Set true with SUGGESTION_TARGET_VERSION = 'X.Y.Z'                  |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                      Pre-Flight Quality Gates                                      |
|  1. cargo fmt -- --check                       (Rust formatting check)                             |
|  2. cargo clippy --all-targets --all-features  (Comprehensive Rust compilation & linter)           |
|  3. npm run build                              (TypeScript compilation & Vite bundle)              |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                               Tagging & Multi-Architecture Packaging                               |
|  - Git Tag format: vX.Y.Z (stable on main) or vX.Y.Z-beta.N (beta on beta)                         |
|  - scripts/before-bundle.js: Merges x86_64 + aarch64 native CLI via lipo into universal agm       |
|  - scripts/package_dmg.sh & Fix_Damaged.command: macOS quarantine removal & DMG assembly           |
|  - src-tauri/hooks.nsh: NSIS installer DisplayName & registry branding                             |
|  - .github/workflows/release.yml: Automated release-target branch gating & asset publishing        |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `scripts/bump-version.mjs` | Multi-manifest atomic synchronizer: updates 14 distinct files with case-insensitive path fallbacks and dual-channel branch detection. |
| `package.json` & `package-lock.json` | Root npm workspace manifests. |
| `version.json` | Canonical version metadata record. |
| `src-tauri/Cargo.toml` & `Cargo.lock` | Rust package definitions and lockfile. |
| `src-tauri/tauri.conf.json` | Tauri v2 desktop application bundle configuration. |
| `Casks/antigravity-tools.rb` | Homebrew Cask formula for macOS automated updates. |
| `src-tauri/hooks.nsh` | NSIS Windows installer registry branding and DisplayName string. |
| `src/components/layout/MiniView.tsx` | MiniView HUD version footer indicator. |
| `src/pages/Settings.tsx` | Settings about tab version display. |
| `changelog.md` & `changelog_en.md` | Dual-language release changelogs with strict attribution discipline. |
| `readme.md` & `readme_en.md` | Primary documentation: stable releases require synchronized release summaries under `## 📝 更新日志` / `## 📝 Changelog`. |
| `src/components/common/SuggestionDeleteThinkingModal.tsx` | Controls upgrade prompt recommending users invalidate stale L2 thinking cache. |
| `scripts/before-bundle.js` | Pre-bundle hook combining x86_64 and aarch64 native CLI binaries into a universal Darwin binary via `lipo`. |
| `scripts/package_dmg.sh` | macOS DMG packaging script bundling `Fix_Damaged.command` for automated Gatekeeper quarantine removal (`xattr -cr`). |
| `.github/workflows/release.yml` | Production release workflow with `verify-release-target` cross-branch interception. |

---

## 3. The 14-Location Atomic Bump Command Matrix

Always invoke version bumping through `npm run bump`:

```bash
# Standard patch bump (e.g., 4.102.2 -> 4.102.3)
npm run bump patch

# Minor feature bump (e.g., 4.102.2 -> 4.103.0)
npm run bump minor

# Major milestone bump (e.g., 4.102.2 -> 5.0.0)
npm run bump major

# Beta preview bump (e.g., 4.102.2 -> 4.103.0-beta.1)
npm run bump beta

# Explicit target version bump
npm run bump 4.103.0
```

### Manifest Targets Checked by `bump-version.mjs`
1. `package.json` (`version`)
2. `package-lock.json` (`version` and `packages[""].version`)
3. `version.json` (`version`)
4. `src-tauri/Cargo.toml` (`package.version`)
5. `src-tauri/Cargo.lock` (`agm-alim` package version)
6. `src-tauri/tauri.conf.json` (`version`)
7. `Casks/antigravity-tools.rb` (`version '...'`)
8. `src-tauri/hooks.nsh` (`VIAddVersionKey "ProductVersion" "..."`)
9. `src/components/layout/MiniView.tsx` (`vX.Y.Z` display label)
10. `src/pages/Settings.tsx` (`vX.Y.Z` display label)
11. `changelog.md` (Inserts release heading template)
12. `changelog_en.md` (Inserts release heading template)
13. `readme.md` (Synchronized for stable releases only)
14. `readme_en.md` (Synchronized for stable releases only)

---

## 4. Release Channel Separation & Invariants

Antigravity-Manager enforces strict branch-to-channel gating:

### 4.1 Stable Releases (正式版)
- **Branch**: Exclusively on `main`.
- **Target Tag**: `vX.Y.Z` (e.g. `v4.102.0`).
- **Behaviors**:
  - Builds official production installers (Windows NSIS/MSI, Linux deb/AppImage, macOS DMG).
  - Updates Docker and GitHub `latest` tags.
  - Services automatic update channels.
  - **MANDATORY**: Synchronize the release summary in both `readme.md` (under `## 📝 更新日志`) and `readme_en.md` (under `## 📝 Changelog`). Never update only `changelog.md` while leaving `readme.md` outdated.

### 4.2 Preview Releases (预览版 / Beta)
- **Branch**: Exclusively on `beta`.
- **Target Tag**: `vX.Y.Z-beta.N` (e.g. `v4.103.0-beta.1`).
- **Behaviors**:
  - Builds independent preview releases (`makeLatest: false`, `prerelease: true`).
  - Does NOT update production update channels or `latest` tags.
  - Does NOT touch `readme.md` or `readme_en.md` (pre-release summaries remain exclusively in `changelog.md` and `changelog_en.md`).

### 4.3 CI Release Gate Interception
In `.github/workflows/release.yml`, the `verify-release-target` job intercepts cross-branch misplacement:
```yaml
- name: Verify release target branch matches tag channel
  run: |
    TAG_NAME="${{ github.ref_name }}"
    BRANCH_NAME="${{ github.base_ref || github.ref_name }}"
    # Enforces stable tags v* (non-beta) must originate from main
    # Enforces beta tags v*-beta.* must originate from beta
```

---

## 5. Governance & Attribution Invariants

### 5.1 Single Contributor Attribution Invariant
- **Rule**: Attribution in `changelog.md` and `changelog_en.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
- **Prohibition**: Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes. This guarantees the GitHub release page contributors list strictly contains `aukgit` and none else.
- Commits from contributors are credited in git commit trailers (`Co-authored-by:`), but release-level public notes preserve singular project attribution.

### 5.2 Case-Insensitive Lowercase Path Resolution
- All repository documentation and scripts must follow lowercase naming conventions (e.g., `changelog.md`, `readme.md`, `release_guide.md`, `pull_request_template.md`).
- `bump-version.mjs` incorporates dynamic `target.relPath.toLowerCase()` resolution to prevent case-sensitive crashes on Linux and macOS CI runners.

---

## 6. Thinking Cache Invalidation Control

File: `src/components/common/SuggestionDeleteThinkingModal.tsx`

When releasing updates that alter proxy thinking formats, database schemas, or prompt caching rules:
1. **Routine Releases (no user prompt)**:
   Keep `SUGGESTION_DELETE_THINKING_STORE = false`.
2. **Major Architecture / Schema Refactors (prompt users once)**:
   1. Set `SUGGESTION_DELETE_THINKING_STORE = true`.
   2. Set `SUGGESTION_TARGET_VERSION = '<version>'` (e.g. `'4.103.0'`).
   3. On upgrade, users with existing thinking cache receive a one-time non-intrusive modal recommending cache cleanup; the action state persists in `gui_config.json`.

---

## 7. Mandatory Pre-Flight Verification Gate

Before committing a release or pushing a release tag, run the following essential checks locally:

```bash
# 1. Check Rust formatting
cd src-tauri && cargo fmt -- --check

# 2. Comprehensive Rust compilation and linter gate (includes all targets and features)
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Compile frontend TypeScript and build bundle
npm run build
```

Full-app compilation (`tauri build`) and multi-platform packaging are handled by GitHub Actions CI runners.

---

## 8. Complete Step-by-Step Release Ceremony Procedure

1. **Verify Clean Working Tree**:
   ```bash
   git status
   ```
2. **Execute Atomic Version Sync**:
   ```bash
   npm run bump <patch|minor|beta|version>
   ```
3. **Audit Changelogs & Attribution**:
   - Inspect Git history: `git log <last-tag>..HEAD --oneline`
   - Fill in `changelog.md` and `changelog_en.md` under the newly generated version header.
   - Confirm all attribution is strictly `@aukgit` (`(Thanks to @aukgit)`).
   - If a stable release, update `readme.md` and `readme_en.md` summaries.
4. **Run Pre-Flight Checks**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   cd src-tauri && cargo clippy --all-targets --all-features
   npm run build
   ```
5. **Commit the Release Manifests**:
   ```bash
   git add .
   git commit -m "chore(release): bump version to X.Y.Z and update changelog"
   ```
6. **Create Tag and Push**:
   ```bash
   # For stable release on main:
   git tag vX.Y.Z
   git push origin main
   git push origin vX.Y.Z

   # For preview release on beta:
   git tag vX.Y.Z-beta.N
   git push origin beta
   git push origin vX.Y.Z-beta.N
   ```
7. **Monitor CI Packaging**:
   Observe `.github/workflows/release.yml` as it compiles binaries, generates DMGs, packages NSIS installers, and publishes the GitHub release.

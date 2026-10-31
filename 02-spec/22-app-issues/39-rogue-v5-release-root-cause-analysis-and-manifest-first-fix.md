# 4-Part Root Cause Analysis: Rogue v5 Release Origin & Manifest-First Resolution

- **Issue ID**: 39
- **Category**: CI/CD & Version Resolution Architecture
- **Severity**: High / Release Channel Contamination
- **Date**: 2026-10-31
- **File**: `02-spec/22-app-issues/39-rogue-v5-release-root-cause-analysis-and-manifest-first-fix.md`
- **Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)

---

## 1. Incident Description & Scope

Operators observed that rogue `v5.0.X` releases (ranging from `v5.0.0` to `v5.0.12`) were published on GitHub Releases and tagged in the repository, despite the canonical production codebase operating on the `v4.283.0` minor bump track. The operator issued the urgent directive: **"Find the rootcuase where did you get v5 version to be released find the root cause and fix it please"**.

---

## 2. Root Cause Analysis (4-Part RCA)

### Part 1: Symptoms & Observability
- Git commit logs revealed a sequence of autonomous bot commits on Saturday Oct 31, 2026:
  - `3659500cf`: `chore(version) - bump to v5.0.12`
  - `411e47ea8`: `chore(version) - bump to v5.0.11`
  - `ae9a0dc43`: `chore(version) - bump to v5.0.9`
  - `4d11b6abb`: `chore(version) - bump to v5.0.7`
  - `15a53d02b`: `chore(version) - bump to v5.0.6`
  - `bfb5d143d`: `chore(version) - bump to v5.0.4`
  - `079e66bda`: `chore(version) - bump to v5.0.3`
  - `7c6b7696f`: `chore(version) - bump to v5.0.2`
- Remarkably, inside those commits, `version.json` and `package.json` were actually modified as `4.279.1` -> `4.279.2` -> `4.279.3`, yet the commit message, release branch, and git tag were named `v5.0.X`.
- The pushed `v5.0.X` git tags triggered `.github/workflows/release.yml`, which automatically compiled binaries and created official GitHub release entries under `v5.0.X`.

### Part 2: Why It Happened (Upstream Origin & Inverted Version Resolution)
1. **Upstream Project Version Leap**:
   - In the upstream open-source project (`lbjlaq` / `Jeik`), commit `0067fe045ddd77f4462ea8e2c25384839d869f32` on Sat Oct 24, 2026 merged a major version jump from `4.9.8` directly to `5.0.0` (`chore(release): bump version to 5.0.0 and update changelog`).
   - When upstream git commits and tags were fetched or merged, git tags `v5.0.0` and `v5.0.1-beta.x` were pulled into the repository's git object store.
2. **Inverted SemVer Detection Priority**:
   - The multi-repository synchronizer script (`03-ai-scripts/38-sync-prompts-skills-scripts.py` and `.agents/scripts/38-sync-prompts-skills-scripts.py`) contained a critical architectural defect in `detect_latest_version(target)`:
     ```python
     # DEFECT: Git tags were queried as Priority 1 before checking version manifests!
     code, out, _ = run_cmd('git tag --list "v[0-9]*" --sort=-v:refname', target)
     ```
   - Because `5 > 4`, the `--sort=-v:refname` query always returned `v5.0.x` at the very top of the list, completely shadowing `version.json` (which was at `4.279.x` / `4.283.x`).
   - `detect_latest_version()` returned `"5.0.2"` and never reached `version.json` or `package.json`.
3. **Runaway Autonomous Loop**:
   - On every execution of `38-sync-prompts-skills-scripts.py`, the script computed `next_ver = bump_patch_version("5.0.2")` -> `"5.0.3"`.
   - It created release branch `release/v5.0.3`, tagged `v5.0.3`, and pushed it to `origin`.
   - On the next sync run, `detect_latest_version()` saw the new `v5.0.3` tag, bumped to `v5.0.4`, and repeated the cycle up to `v5.0.12`.

### Part 3: Where in the Codebase It Originated
1. `03-ai-scripts/38-sync-prompts-skills-scripts.py:403`:
   - Prioritized `git tag` query over `version.json`, violating the canonical repository rule that `version.json` is the sole source of truth.
2. `.agents/scripts/38-sync-prompts-skills-scripts.py:403`:
   - Duplicate copy in `.agents/scripts/` exhibiting identical tag-first resolution logic.
3. `.github/workflows/release.yml:45-56`:
   - When a tag matching `v*` was pushed, the CI gate checked only regex syntax `^v[0-9]+\.[0-9]+\.[0-9]+` and did not assert that the tag matched the authoritative version declared in `package.json` or `version.json`.

### Part 4: Surgical Remediation & Prevention
1. **Manifest-First Version Resolution**:
   - Inverted `detect_latest_version(target)` in both `03-ai-scripts/38-sync-prompts-skills-scripts.py` and `.agents/scripts/38-sync-prompts-skills-scripts.py`:
     - **Priority 1**: `version.json` (canonical source of truth per `version.json` specification).
     - **Priority 2**: `package.json`.
     - **Priority 3**: `version.txt` / `VERSION`.
     - **Priority 4 (Fallback only)**: `git tag` — strictly if NO version manifest file exists.
   - When a manifest exists, rogue git tags can NEVER override the declared version.
2. **Release Workflow Version Gate**:
   - Hardened `.github/workflows/release.yml` with Step 1.5:
     ```bash
     CANONICAL_VERSION=$(jq -r '.version' package.json 2>/dev/null || jq -r '.Version' version.json 2>/dev/null)
     if [ -n "$CANONICAL_VERSION" ] && [ "$CANONICAL_VERSION" != "null" ]; then
       EXPECTED_TAG_PREFIX="v${CANONICAL_VERSION}"
       if [[ "$TAG_NAME" != "$EXPECTED_TAG_PREFIX"* ]]; then
         echo "::error::❌ Tag '$TAG_NAME' does not match repository package.json/version.json version ('$CANONICAL_VERSION')!"
         exit 1
       fi
     fi
     ```
   - Any tag diverging from `version.json` is fail-stopped immediately before building or publishing.
3. **Complete Purge of Stale Tags & Releases**:
   - Deleted all 7 remote GitHub releases matching `v5*` (`v5.0.1` through `v5.0.7`) via `gh release delete`.
   - Deleted all 5 untagged remote git tags (`v5.0.8` through `v5.0.12`) via `git push origin --delete`.
   - Deleted all 31 local git tags matching `v5*` via `git tag -d`.
   - Verified that both local and remote git tag queries for `v5*` return exactly 0 results.

---

## 3. Verification & Invariant Proof

- `git ls-remote --tags origin "v5*"`: **0 results** (clean).
- `git tag -l "v5*"`: **0 results** (clean).
- Canonical production release: `v4.283.0` (commit `297cfe042`).
- Synchronizer script version detection: Reads `4.283.0` directly from `version.json`.

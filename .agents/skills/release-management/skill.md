---
name: release-management
description: Execute full release ceremony, SemVer version bumps, package synchronization, and changelog updates.
---

# Release Management

Conducts the standardized release ceremony for coding-guidelines-v24.

## Steps
1. Determine bump tier (MINOR default, reset PATCH to 0).
2. Update version in `package.json` and run `npm run sync`.
3. Prepend entry to `changelog.md` with verified, concrete changes.
4. Commit: `chore(release): bump version to X.Y.0`.
5. DO NOT create manual git tag if managed externally.
6. Push to remote.

# Subtask 04 - Bump, tag, release (244)

**Date**: 01-Nov-26
**Status**: TODO at plan time; completed after the fix commit is CI-green

- Bump: `npm run bump minor` (no `--commit` flag; commit via gitmap).
  Expected: 4.287.0 -> 4.288.0. Verify the diff before committing
  (changelog attribution strictly `@aukgit`; stable-release README
  changelog sync required).
- Tag: `v4.288.0` on the bump commit, pushed with raw git
  (`git tag v4.288.0`, `git push origin v4.288.0`). Declared raw-git
  exception: `gitmap release` creates a release branch plus metadata
  violating the main-only convention, and `gitmap push` treats its first
  argument as a repo name and errors on a tag.
- Never: move `v4.286.0` / `v4.287.0`, re-bump 4.284.0 / 4.285.0,
  force-push, or change `.github/workflows/` files.
- Release workflow is tag-triggered with a gate-first CI-green check;
  watch it to success.

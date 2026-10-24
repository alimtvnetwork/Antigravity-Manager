# RCA 47: Release Gate Premature CI Conclusion Race Condition

- **Date**: 2026-10-24
- **Commit**: `336ef23e` / `7faf63d6`
- **Release Runs**: `#40103662017` (v4.263.0), `#40100454678` (v4.262.0), `#40098753774` (v4.261.0), `#40097853507` (v4.260.0)
- **Component**: `.github/workflows/release.yml` (`verify-release-target` -> `Release gate - CI must be green for this commit`)

---

## Part 1: Symptoms & Error Signature

When pushing a tag simultaneously with release commits to `main`, the `Release` workflow fails at job `Verify Branch & Release Type Compatibility`, step `Release gate - CI must be green for this commit` within 13–15 seconds of kickoff:

```text
Latest completed CI run for 336ef23e94630b14599ca2e090de84d913359f1b concluded: missing
##[error]❌ [Release Gate Intercept] CI for commit 336ef23e94630b14599ca2e090de84d913359f1b concluded 'missing', not 'success'. Release refused.
##[error]Process completed with exit code 1.
```

However, the parallel `CI` workflow for that commit (`#40103630993`) ran normally, was actively in progress, and finished successfully ~8–10 minutes later with 100% green pass.

---

## Part 2: Root Cause

In `.github/workflows/release.yml`, the release gate verification query was:

```bash
CONCLUSION=$(gh api "repos/${{ github.repository }}/actions/workflows/ci.yml/runs?head_sha=${{ github.sha }}&status=completed&per_page=1" --jq '.workflow_runs[0].conclusion // "missing"')
if [ "$CONCLUSION" != "success" ]; then
  echo "::error::❌ [Release Gate Intercept] CI for commit ${{ github.sha }} concluded '$CONCLUSION', not 'success'. Release refused."
  exit 1
fi
```

1. **Premature Evaluation Race Condition**: `release.yml` triggers immediately on tag push (`v*`), starting within 10–15 seconds. At that exact time, `ci.yml` (triggered by the commit push on `main`) is either in status `queued` or `in_progress`.
2. Because the API query filtered by `status=completed` without any polling or wait loop, it found zero completed runs and defaulted to `missing`.
3. The check immediately exited with code 1 without waiting for the in-flight CI run to finish.
4. **Main Concurrency Group Collision**: In `ci.yml`, concurrency was grouped by `CI-${{ github.ref }}` (`CI-refs/heads/main`), causing subsequent commits on `main` to cancel previous in-flight runs on `main`.

---

## Part 3: Surgical Fix

1. **Polling Wait Loop in Release Gate**: In `.github/workflows/release.yml`:
   - Replaced the single-shot query with a bounded polling loop (`MAX_WAIT_SECONDS=3600`, polling every `POLL_INTERVAL=20`s).
   - Queries `actions/workflows/ci.yml/runs?head_sha=${{ github.sha }}&per_page=1` without filtering by `status=completed`.
   - If no run has registered yet, waits up to 90s for GitHub's event registration.
   - If status is `in_progress` or `queued`, logs progress and waits for completion.
   - Once `status=completed`, evaluates `conclusion`: if `success`, allows the release to proceed; if failure or cancelled, fails fast.
   - Times out safely after 60 minutes if CI never completes (sized generously to absorb runner queue delays).
2. **Main Branch Concurrency Isolation**: In `.github/workflows/ci.yml`:
   - Scoped concurrency group on `main` to the specific commit SHA (`group: ${{ github.workflow }}-${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}`), ensuring runs on `main` are never cancelled by subsequent commits while preserving auto-cancellation for PR branches.
   - If `success`: allows the release to proceed.
   - If `failure`, `cancelled`, or anything else: fails fast.

---

## Part 4: Verification

1. Verified bash script extraction logic with `gh api` and `@tsv` parser against commit `336ef23e94630b14599ca2e090de84d913359f1b`, confirming it detected `completed` status and `success` conclusion.
2. Re-ran release run `#40103662017` on GitHub Actions, which passed `Verify Branch & Release Type Compatibility` in 23s and proceeded to build multi-platform Tauri release assets.
3. Released version `v4.264.0` containing this workflow fix.

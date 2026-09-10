# Promoting checks to REQUIRED on `main`

Turn a passing workflow into a merge blocker on `main`. Backlog item 1 (promote `slides-visual` to required) tracks the two remaining slides checks.

## Preconditions

- Repo admin permission on GitHub.
- `gh` CLI logged in as an admin: `gh auth status`.
- The workflow has been green on `main` for at least one full release cycle. Promoting a flaky check blocks every PR.
- **Readiness gate (v5.128+):** run `npm run branch-protection:soak` and confirm the target check reports `READY` (pass rate >=95% over the last 20 runs on `main`). Do NOT promote a `WATCH` or `NOT READY` check. If output is `NO DATA`, the workflow has not run on `main` at all; trigger a run first.

## Procedure

0. Confirm the target check is stable.
   ```bash
   npm run branch-protection:soak
   ```
   Every desired entry must print `READY`. If any prints `WATCH` / `NOT READY` / `NO DATA`, stop and fix that first, do not promote.

1. Regenerate the machine-readable payload from the checked-in expectation file.
   ```bash
   node scripts/print-required-checks.mjs --json
   ```
   Copy the `contexts` array from the output.

2. Add the new check name to `.github/branch-protection.expected.json` under `required`, and remove it from `desired-but-not-yet-required` if present. Commit that change with the same PR that flips branch protection.

3. Apply the branch-protection change. Replace `{owner}/{repo}` and paste the fresh contexts array.
   ```bash
   gh api -X PATCH "repos/{owner}/{repo}/branches/main/protection/required_status_checks" \
     -f strict=true \
     -F 'contexts=["Linter Scripts Validation","Codegen (Rule 9 inverted-fields)","Spec Cross-Reference Validation","Sync Drift Check (all generated files)","Validate version.json schema","visual"]'
   ```
   `strict=true` means PRs must be up-to-date with `main` before merge (blocks stale-branch bypasses).

4. Verify (v5.128+): use the one-command diff instead of eyeballing raw JSON.
   ```bash
   npm run branch-protection:diff
   ```
   Exit 0 means live state matches `.github/branch-protection.expected.json` exactly. Exit 1 prints the drift (missing/extra contexts) with a clear diff. Fallback for older setups:
   ```bash
   gh api "repos/{owner}/{repo}/branches/main/protection/required_status_checks" \
     --jq '.contexts'
   ```

5. Re-run the enumerator to confirm no drift.
   ```bash
   node scripts/print-required-checks.mjs --check
   ```

## Rolling back a required check

If a newly-required check turns flaky and starts blocking merges:

```bash
gh api -X PATCH "repos/{owner}/{repo}/branches/main/protection/required_status_checks" \
  -f strict=true \
  -F 'contexts=[ ... same list minus the flaky check ... ]'
```

Then move the check back to `desired-but-not-yet-required` in `.github/branch-protection.expected.json` with a `reason` explaining what broke and what has to be true before re-promotion.

## Why this file exists

The exact `gh api` payload was never captured, so the "promote to required" backlog item kept getting deferred every release. Anyone with admin can now execute this in under a minute.

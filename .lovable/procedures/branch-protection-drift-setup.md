# Nightly Branch-Protection Drift Job: Setup Checklist

One-time setup so `.github/workflows/branch-protection-drift.yml` runs green every night without surprising an admin at 3am.

## Why a PAT (not `GITHUB_TOKEN`)

The default `GITHUB_TOKEN` cannot read `repos/{owner}/{repo}/branches/main/protection`. That endpoint requires an admin-scoped token. The workflow therefore reads a repo secret named `GH_ADMIN_TOKEN`. Rename in one place only: the workflow env block.

## Admin PAT permissions

Create a fine-grained PAT owned by a maintainer account (or, preferably, a dedicated bot account with admin on this repo).

Fine-grained PAT (preferred):
- [ ] Resource owner: the org/user that owns this repo.
- [ ] Repository access: **Only select repositories** -> pick this repo only.
- [ ] Repository permissions:
  - [ ] `Administration`: **Read-only** (required for branch-protection reads).
  - [ ] `Contents`: **Read-only** (for `actions/checkout`).
  - [ ] `Metadata`: **Read-only** (auto-selected).
  - [ ] `Issues`: **Read and write** (only if you enable auto-file-issue-on-drift; otherwise omit).
- [ ] Expiration: <=90 days. Calendar reminder to rotate.
- [ ] Do NOT grant: `Actions: write`, `Workflows: write`, `Secrets`, `Packages`, `Pull requests: write`. Drift job is read-only.

Classic PAT (fallback only if fine-grained is blocked by org policy):
- [ ] `repo` (full) is the minimum that unlocks branch-protection reads.
- [ ] No `admin:org`, no `delete_repo`, no `workflow`.

## Repository secrets

Settings -> Secrets and variables -> Actions -> **Repository secrets**:
- [ ] `GH_ADMIN_TOKEN` = the PAT value above. Repo-scoped, not environment-scoped (nightly cron has no environment).
- [ ] Confirm no same-named secret exists at org level that could shadow it with a stale value.

## Governance guardrails

- [ ] PAT owner is documented in `.lovable/procedures/branch-protection.md` "owner" line so rotation is not orphaned when the maintainer leaves.
- [ ] Rotation cadence recorded (<=90 days). Add to team calendar.
- [ ] On rotation: update `GH_ADMIN_TOKEN` first, then re-run the workflow manually (`gh workflow run branch-protection-drift.yml`) to confirm green before the old PAT is revoked.
- [ ] Branch protection on `main` explicitly includes the drift job's own check name **only after** it has soaked READY (see `branch-protection.md`), never before, or a PAT outage locks the repo.
- [ ] Alert routing: workflow failure notifications go to a shared channel, not one person's inbox.

## Smoke test

Run once after wiring:
```bash
gh workflow run branch-protection-drift.yml
gh run watch
```
Expect exit 0 and log line `live state matches expected`. If it fails with 403 on `/branches/main/protection`, the PAT is missing `Administration: read`. If it fails on checkout, `Contents: read` is missing.

## Rollback

If the PAT is compromised or the job misfires:
1. Revoke the PAT in GitHub Developer Settings (immediate).
2. Delete `GH_ADMIN_TOKEN` in repo secrets.
3. Nightly job will fail loud (auth error), not silently pass. That is the intended failure mode.

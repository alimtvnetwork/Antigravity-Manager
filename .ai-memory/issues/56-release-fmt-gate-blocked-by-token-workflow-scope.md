# Issue 56: Release rustfmt gate cannot be pushed (token lacks `workflow` scope)

Status: resolved 2026-10-01 (`6b18ddc4` on `origin/main`, pushed after the maintainer granted the `workflow` scope)
Raised: 2026-10-01
RCA: [cicd-issues/40](../cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md)
Spec: [02-spec/21-app/89-cicd-rustfmt-cmd-instances-fix.md](../../02-spec/21-app/89-cicd-rustfmt-cmd-instances-fix.md)

## 1. Symptom
`git push origin main` is rejected:

```text
! [remote rejected] main -> main (refusing to allow a Personal Access Token to create or update workflow `.github/workflows/release.yml` without `workflow` scope)
```

The local commit `ci(release): gate release on cargo fmt --check` stays unpushed.

## 2. Trigger
Any push whose commits touch `.github/workflows/*`.

## 3. Root cause
Both credentials on the maintainer host carry only `gist`, `read:org`, `repo`: the `gh` OAuth token (keyring) and the Git Credential Manager token. GitHub requires the `workflow` scope to create or modify workflow files. Adding a scope needs an interactive browser authorization that an agent cannot complete.

## 4. Impact
- `release.yml` still publishes from tags on commits with red CI (the failure mode that shipped `v4.109.1` to `v4.109.4`).
- The announced CI deadlines in RCA 40 section 7 (Node 24 action majors, Ubuntu 26 runner migration on 2026-10-19) also need workflow edits and are blocked the same way.

## 5. Fix (maintainer action, once)
```powershell
gh auth refresh -h github.com -s workflow
gh auth setup-git
```
Then push the pending local commit: `git push origin main`.

## 6. Prevention
- Keep workflow edits in their own commit so the rest of a fix can still ship with a `repo`-only token.
- Before promising a workflow change, check `gh auth status` for `workflow` in "Token scopes".

## 7. Close criteria
`git log origin/main -- .github/workflows/release.yml` shows the rustfmt gate step `Release gate - Rust formatting must match CI`.

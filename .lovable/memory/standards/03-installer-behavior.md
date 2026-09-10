---
name: Generic Installer Behavior
description: Two-mode installer contract — PINNED (exact tag, no fallback, no cross-repo) vs IMPLICIT (latest → V→V+20 parallel discovery → main). Repo-agnostic; canonical spec at 02-spec/14-update/27-generic-installer-behavior.md.
type: standard
---

# Generic Installer Behavior (Cross-Repo Standard)

**Canonical spec:** `02-spec/14-update/27-generic-installer-behavior.md` v1.0.0
**Plan:** `.lovable/plans/01-index.md`

Applies to EVERY installer in EVERY repo: `install.*`, `release-install.*`,
`quick-install.*`, `error-manage-install.*`, `linters-install.*`,
`slides-install.*`, `wp-install.*`, `cli-install.*`, `splitdb-install.*`,
`consolidated-install.*`, etc.

## Two modes — chosen at startup, never switched mid-run

### PINNED (strict version supplied via `--version` / `-Version`, env, baked-in placeholder, or `/releases/download/<tag>/` URL)

- MUST install exactly that tag.
- MUST NOT query `releases/latest`.
- MUST NOT fall back to `main`.
- MUST NOT cross repo boundaries (no `repo-v15 → repo-v16` jump).
- MUST NOT pick "compatible" or "nearest" version.
- 404 on the pinned tag → exit code `3`. No silent downgrade.

### IMPLICIT (no version supplied)

Source order, stop at first success:
1. Latest release of current repo.
2. **V → V+20 parallel discovery** (only if repo name matches `{repo-base}-v{N}`).
3. Main branch tarball with visible `⚠️ unstable` warning.

## V → V+N discovery rules

- Default `LOOKAHEAD = 20` (was 5 in older `24-update-check-mechanism/02-fundamentals.md`; new spec supersedes).
- Probes fire in parallel (Bash `&`+`wait`, PS `Start-Job`, Node `Promise.all`, Go goroutines, Python `asyncio.gather`).
- 5s per probe, 10s total deadline, 0 retries.
- Highest existing `K` wins.
- **Forbidden in PINNED MODE.**

## Exit codes (normative)

`0` success · `1` generic · `2` offline blocked · `3` pinned tag not found · `4` verification failed · `5` handoff rejected. Repo-specific codes ≥ `10`.

## Required banner

```
📦 {InstallerName} {Version}
   mode:    pinned | implicit
   repo:    {owner}/{repo}
   version: {tag | "discovering…" | "main"}
   source:  release-asset | tag-tarball | main-branch | local-archive
```

## How to apply to a new repo

Hand the AI just `02-spec/14-update/27-generic-installer-behavior.md`. It is
self-contained, repo-agnostic (uses `{owner}`, `{repo-base}`, `{N}`), and
§11 contains the step-by-step adoption checklist.

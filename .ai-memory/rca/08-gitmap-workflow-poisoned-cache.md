# RCA-08 — re-running CI on a failed SHA short-circuits everything: the `test-summary` job's `if: success()` mark step poisoned the `ci-passed-<sha>` cache

Repo: `alimtvnetwork/gitmap-v28` · CI workflow: the SHA-based
already-built short-circuit (`ci-passed-<sha>` cache) · Writer:
`.github/actions/mark-sha-built/action.yml` (invoked from the
`test-summary` job) · Standing rule: NEVER edit `.github/workflows/`
(the recommended fix below is documented only, NOT implemented)

## Symptom

Re-running CI for a commit SHA that had **failed** did not re-run the
pipeline — every job short-circuited instantly and the run went green.
The log of the SHA-check job showed:

```
Cache hit for: ci-passed-<sha>
```

…setting `already-built == 'true'`, which every downstream job used to
skip its real work. The SHA had never passed CI, yet the cache said it
had. The dangerous end state: a maintainer sees a green run, concludes
the SHA is fine, and merges or releases code that is still broken. The
"green" is a lie written by the pipeline itself.

## Root cause

**One line:** the `test-summary` job runs `if: always()` and its "Mark
SHA as built" step uses `if: success()` — which checks only the *job's
own steps*, not the pipeline — so a failed pipeline still writes the
"passed" cache entry.

Step by step, for zero-context readers:

1. gitmap CI has an already-built optimization: the first job checks the
   GitHub Actions cache for the key `ci-passed-<sha>`. On a hit it sets
   the output `already-built=true`, and every expensive downstream job
   (build, test, E2E) skips itself with
   `if: needs.<sha-check>.outputs.already-built != 'true'`. On a miss the
   pipeline runs normally, and at the end the `test-summary` job records
   the result by writing the `ci-passed-<sha>` cache key (via
   `.github/actions/mark-sha-built/action.yml`). The cache is the
   pipeline's memory of "this SHA already passed".
2. The `test-summary` job is declared with `if: always()` — it runs even
   when other jobs failed — which is correct for a *summary*. The bug is
   one level down: its inner "Mark SHA as built" step is gated on
   `if: success()`.
3. In GitHub Actions, a step-level `if: success()` means **"all previous
   steps in this job succeeded"**. It does NOT mean "the pipeline
   succeeded" and it does NOT mean "my `needs:` jobs succeeded". Because
   the `test-summary` job ran under `if: always()`, it always *reaches*
   the mark step; and because that step's own predecessors (in its own
   job) succeeded, `success()` evaluates to **true** — even when the
   build and test jobs it summarizes are red.
4. So every pipeline run — green or red — wrote `ci-passed-<sha>` into
   the cache. The cache key is unconditionally poisoned: it no longer
   means "passed", it means "ran". Any re-run of that SHA then sees a
   cache hit and skips everything.

The bug class is "scope confusion in a condition": `success()` reads
naturally as "everything is fine", but its actual scope is the current
job's steps only. Inside an `if: always()` job, `success()` is nearly
always true — the combination turns an opt-in marker into an
unconditional one.

## How it was found (exact commands)

All commands from the repo root (`~/workspace/repos/gitmap-v28`).
`gh run view` reads CI logs from GitHub (read-only); `gitmap aum search`
is the house codebase-search command.

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28

# 1. A re-run of a known-failing SHA went green in seconds instead of
#    minutes. Suspicious speed = something skipped. Pull the log of the
#    SHA-check job (replace <sha-check> with the job id from `gh run view`).
gh run view --job <sha-check> --log
#    -> "Cache hit for: ci-passed-<sha>"
#    -> already-built == 'true' — everything else skipped itself

# 2. The SHA never passed, so who wrote the "passed" key?
#    Search the repo for the writer (no grep allowed in this house;
#    gitmap search only).
gitmap aum search 'ci-passed' .
#    -> hits in .github/actions/mark-sha-built/action.yml (the writer)
#       and the SHA-check job (the reader)

# 3. Read the writer and its caller: the "Mark SHA as built" step sits in
#    the test-summary job with `if: always()` on the job and
#    `if: success()` on the step.
gitmap cat .github/actions/mark-sha-built/action.yml
#    -> step-level `if: success()`: true whenever the job's own previous
#       steps succeeded — regardless of the red build/test jobs.
```

Step 1 names the lie (cache hit for a SHA that never passed); steps 2–3
trace the writer and its condition, which is the whole diagnosis.

## Workaround used

Delete the poisoned cache entry and re-run, forcing the pipeline to do
its real work:

```bash
gh cache delete "ci-passed-<sha>"
```

This is a one-off repair, not a fix: the next red run re-poisons the key
again. It is only safe because the standing rule forbids the real fix.

## Recommended fix — NOT implemented (standing no-workflow-changes rule)

**Never edit `.github/workflows/` or `.github/actions/` — this section is
documentation only.** The authorized fix (for whoever is allowed to touch
the workflows) is to gate the "Mark SHA as built" step on the pipeline's
actual result instead of the job's own steps. Concretely, replace the
step's `if: success()` with a condition over the needed jobs' outcomes,
e.g.:

```yaml
if: ${{ needs.build.result == 'success' && needs.test.result == 'success' }}
```

(or whatever the full set of required jobs is), so the `ci-passed-<sha>`
key is written only when the pipeline genuinely passed. Two design notes
for the implementer:

- Do not rely on `success()` anywhere inside an `if: always()` job —
  that combination is the exact defect. `success()` is job-scoped, always
  was, always will be.
- Alternatively, write the marker from a dedicated job whose `needs:`
  list is the full green set and whose own `if:` is omitted entirely
  (default = all needed jobs succeeded). That makes the "only mark on
  green" invariant structural rather than conditional.

## Prevention — checklist item

**Never trust a green short-circuit — verify `already-built` semantics
before re-running or merging:**

1. If a re-run of a SHA finishes suspiciously fast (seconds, not
   minutes), open the SHA-check job log and look for
   `Cache hit for: ci-passed-<sha>`. A hit means *nothing ran* — the
   green is the cache talking, not the code.
2. Before merging on the strength of a re-run, confirm the cache key
   corresponds to a genuinely green pipeline run *of that exact SHA*
   (check the `gh run view` history for the SHA, not just the latest
   run's badge).
3. When adding or reviewing any cache-write step in CI, ask: "what is
   the exact condition under which this key is written, and what does a
   reader of the key *assume* it means?" If the write condition is weaker
   than the reader's assumption (`ran` vs `passed`), the cache is a
   time bomb. Write the invariant in a comment next to the step.

One-line rule for the dummy AI: **`if: success()` inside an
`if: always()` job is always true — it can never mean "the pipeline
passed".** Any cache key written under that condition will poison every
future re-run of the SHA.

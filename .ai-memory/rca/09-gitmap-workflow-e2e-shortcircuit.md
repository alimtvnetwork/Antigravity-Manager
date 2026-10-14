# RCA-09 — E2E jobs fail on re-run at step "Already validated": bash cannot start because the skipped checkout never created `cli/`

Repo: `alimtvnetwork/gitmap-v28` · CI workflow: `.github/workflows/ci.yml`
(E2E jobs) · Failing step: `Already validated` · Standing rule: NEVER
edit `.github/workflows/` (the recommended fix below is documented only,
NOT implemented)

## Symptom

On a re-run where the SHA-check short-circuit fired (`already-built ==
'true'`), the E2E jobs failed at the step named **"Already validated"**
with:

```
An error occurred trying to start process '/bin/bash' with working
directory '.../cli'. No such file or directory
```

The step that failed is the do-nothing no-op step — its entire body is
essentially "print a message and skip". It fails not because of anything
it *does*, but because of where the runner tries to *start it*. The
short-circuit path — the path designed to be the fast, always-green one —
is the one that errors.

## Root cause

**One line:** the E2E job sets
`defaults.run.working-directory: cli`, but when `already-built ==
'true'` the checkout step is skipped, so `cli/` never exists and bash
cannot start in it.

Step by step, for zero-context readers:

1. In GitHub Actions, a job-level `defaults.run.working-directory: cli`
   applies to **every** `run:` step in that job: the runner starts each
   step's shell with its current directory set to `cli/` (relative to the
   workspace). It is a convenience so build/test steps do not each repeat
   `working-directory: cli`.
2. The already-built short-circuit works by *skipping steps*: when the
   SHA-check job reports `already-built == 'true'`, the E2E job's
   checkout step is skipped (`if: already-built != 'true'` on the
   checkout), and the job instead runs the single no-op step "Already
   validated" to record the skip.
3. Skipping the checkout means the workspace is empty — there is no
   `cli/` directory on the runner at all.
4. But the job-level `defaults.run.working-directory: cli` still applies
   to the no-op step. The runner dutifully tries to launch `/bin/bash`
   with cwd `<workspace>/cli`, which does not exist, and the process
   start itself fails — before a single line of the step's script runs.

The bug class is "a convenience default that outlives its precondition":
`working-directory: cli` is only valid when the checkout happened, but
the default is declared unconditionally at job scope, so the one step
that exists precisely for the no-checkout path inherits a precondition
it cannot satisfy.

Compounding factor: this bug is a sibling of RCA-08. The poisoned
`ci-passed-<sha>` cache (bug 08) sends *more* re-runs down the
already-built path — so the cache poison makes the always-failing
short-circuit step fire more often. The two defects multiply each other:
a broken SHA gets "green" by skipping (bug 08), and the skip path itself
errors (bug 09).

## How it was found

1. **CI log.** The re-run's E2E jobs were red at the "Already validated"
   step — the step with no real logic. The error named the mechanism
   directly: the runner could not *start bash* with cwd `.../cli`.
   When a no-op step fails at process start, the problem is the
   environment the runner built for it, not the step.
2. **Read the E2E job definition** in `.github/workflows/ci.yml` (read
   only — the standing rule forbids edits):
   - job-level `defaults.run.working-directory: cli` — present;
   - checkout step gated on `already-built != 'true'` — skipped on
     re-run;
   - the "Already validated" no-op step with no `working-directory`
     override — inherits the job default.
   
   The three facts together are the diagnosis: on the skip path the
   directory the default points at does not exist.

Verification-by-reading (the only verification the standing rule
allows): every other `run:` step in the E2E jobs assumes a checked-out
tree (they need the checkout to have run), so only the "Already
validated" step — the single step that runs *without* a checkout —
collides with the job-level default. There is exactly one step that can
hit this, and it is the one that did.

## Workaround used

None available without touching the workflow — the defect is structural
in the YAML, and the standing rule blocks the only durable repairs. The
failure was documented as-is (this RCA) so that whoever is authorized to
edit the workflows can apply the fix below. Deleting the poisoned cache
(see RCA-08's `gh cache delete "ci-passed-<sha>"`) incidentally avoids
the failing step by forcing a full checkout, but that treats the
symptom of bug 08, not the defect of bug 09.

## Recommended fix — NOT implemented (standing no-workflow-changes rule)

**Never edit `.github/workflows/` — this section is documentation only.**
The authorized fix (for whoever may edit the workflows) is to stop
applying the `cli` working directory to the no-checkout step. Two
equivalent options, either is fine:

- **Option A (minimal):** give the "Already validated" step its own
  `working-directory` override pointing at the workspace root (e.g.
  `${{ github.workspace }}`), which always exists.
- **Option B (structural):** remove the job-level
  `defaults.run.working-directory: cli` and set `working-directory:
  cli` explicitly on each step that needs the checkout. More verbose,
  but no step can ever inherit a directory precondition it did not ask
  for.

Option B is the more future-proof: any *new* no-checkout step added
later would silently inherit the job default under Option A-style
thinking. But Option A is the smaller diff. Either way, the invariant
to preserve: **no step may run with a working directory that only
exists on the checkout path unless that step also requires the
checkout.**

## Prevention — checklist item

**Every short-circuit / skip path must be tested, not just the happy
path:**

1. When a workflow gains a skip condition (`already-built`, cache-hit
   fast paths, `[skip ci]`, manual `workflow_dispatch` inputs that skip
   steps), run the workflow *in the skipped state* at least once and
   read every step's log — including the no-op "already done" steps.
   The happy path (full checkout, everything present) cannot reveal
   defects that only exist when steps are absent.
2. Treat job-level `defaults.run` as a precondition contract: list the
   steps it applies to, then ask for each one, "does this step's
   directory exist in *every* combination of skip conditions?" If any
   answer is no, the default is wrong for that step.
3. In review, be suspicious of any no-op step that inherits job-level
   defaults — a step that "does nothing" still needs a place to stand.

One-line rule for the dummy AI: **a job-level `working-directory` is a
promise that the directory exists for *every* step — including the
no-op step that only runs when the checkout was skipped.**

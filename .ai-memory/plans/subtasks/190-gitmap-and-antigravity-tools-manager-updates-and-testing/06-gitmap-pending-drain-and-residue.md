# Subtask 06 — gitmap pending drain + CG residue sweeps

Repo: gitmap-v28. Spec: 02-gitmap-cg-execute-spec.md steps 1 and 5.
Status: PENDING. Owner: Worker wave 2.

## Goal
The pending folder tells the truth, and the first CG campaign's residue is closed repo-wide.

## Steps
1. Move the 24 completed plans (67, 75, 287-294, 296-309) from `.ai-memory/plans/pending/` to `.ai-memory/plans/completed/` (create the folder if missing, following the repo's existing convention). Append a one-line closure note to each. Leave 01 and 56 in pending: append a re-triage note to 01 (predates the pure-Go nodes rewrite — verify-or-close in a later task) and a go/no-go note to 56.
2. Boolean residue sweep (repo-wide, linter-driven — not file boxes): eliminate all explicit `== true` / `== false` comparisons in production Go code (109 hits at audit; known sites cli/cmd/root.go, cli/cmdagy/agy_fix_pipeline_exec.go, cli/cmdagy/agy_fix_pipeline_inject.go). Use `gitmap aum search` to enumerate, fix each by direct condition use. Do NOT touch the deliberate `IsNotNull` API field. Fix the `isNotFound` in cli/tests/e2e/prompt_repo_e2e_test.go.
3. `fmt.Errorf` → AppError continuation: continue the migration in the highest-traffic packages first (cli/cmd*, cli/store) following the established apperror/appfault patterns from plans 296-301. This is a continuation, not a forced 100%: convert where the error crosses a function boundary as a return value; leave `%w`-wrapped internal errors that never surface. Record the before/after hit counts in the completion note.
4. Update .ai-memory/plans/readme.md entries for the moved plans (or hand the list to the lead if the index is lead-owned).

## Rules
- NO builds, NO test runs (user instruction). Verification is subtask 08 + CI.
- GitMap-only search. Relative paths only.

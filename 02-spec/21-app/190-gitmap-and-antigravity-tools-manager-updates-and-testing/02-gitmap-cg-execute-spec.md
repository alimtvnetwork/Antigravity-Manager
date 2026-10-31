# 02 — GitMap Code Quality + CG Execute Zero-to-End Spec

Slug: 190-gitmap-and-antigravity-tools-manager-updates-and-testing
Scope: gitmap-v28. Companion specs: 01 (AGM restore), 03 (E2E + release).

## State at HEAD 8b1dd0743 (measured)

- Pending folder: 26 plans, **24 substantively complete** but never drained;
  genuinely open: 01-ports-and-ssh-enablement (likely obsolete — predates the
  pure-Go nodes rewrite; re-triage before effort) and
  56-vmware-hardware-batch-and-macro-orchestration (hardware-dependent,
  needs explicit go/no-go).
- CG execute suite (`01-prompts/15-cg-execute/`, prompts 01–39): 12 campaigned
  (02–11, 19, 20), 2 partially overlapped (16, 18), ~25 untouched.
- Residue from scoped file-box waves (repo-wide counts):
  - explicit `== true` / `== false` comparisons: **109 hits** (incl.
    `cli/cmd/root.go:111`, `cli/cmdagy/agy_fix_pipeline_exec.go:60`,
    `agy_fix_pipeline_inject.go:193,208,247`)
  - bare `fmt.Errorf`: **1,506 hits** (apperror 6,968 / appfault 952 — the
    migration is partial)
  - negative boolean names: 3 hits (1 deliberate API field, 1 e2e test)
  - `result.Result[` stutter: 1 hit (result package's own test)
- Function-size guideline (prompt 22) NOT started: `cli/cmd/root.go`
  `dispatch` = 116 lines, `Run` = 99; `cli/store/agent_store.go`
  `InitTaskDB` = 86.

## Required execution order (blast-radius sequenced)

1. **Drain the pending folder**: move the 24 completed plans to
   `.ai-memory/plans/completed/`; re-triage 01; park 56 pending go/no-go.
2. **Prompt 24** (isolate destructive OS / heavy unit tests) — safety net.
3. **Prompts 12, 13** (relative paths; CLI commands & help) — mechanical.
4. **Prompts 14 → 18 → 22** (function signatures → argument reduction →
   file-size & function reduction) — attacks the 116-line dispatch class.
5. **Repo-wide residue sweeps** (linter-driven, NOT per-file boxes):
   `== true/false` elimination (109), `fmt.Errorf` → AppError continuation,
   negative-name cleanup (3).
6. **Prompts 29 → 25** (DRY / library extraction → nuclear modularization).
7. **Efficiency**: 21 (lazy regex), 23 (string ops), 37 (EqualFoldAny),
   38 (map dispatch), 36 (variadic), 34 + 39 (pointer/value semantics).
8. **Language surfaces**: 15 (TypeScript), 17 (terminal UI), 16 remainder,
   26/27/28 (Python trio — `03-ai-scripts/` untouched so far), 33 (branch
   immutability), 30 (clean build & caches), 35 (clean artifacts).
9. **Meta**: 31, 32, then prompt 01 as the final verification sweep.

## Rules

- NO builds, NO test runs during implementation (user instruction). All
  verification happens in spec 03's end phase + CI.
- Search exclusively via GitMap. Relative paths only. Lowercase filenames.
- Closure is linter-proven, not claim-proven: boolean linter,
  `check-newline-styling.py --all`, `check-enum-guidelines.py`, and FastGate
  (`03-ai-scripts/50-fastgate.py`) must all report clean repo-wide at the end.
- Follow each CG prompt's own text in `01-prompts/15-cg-execute/`; the plan
  series 296–309 in `.ai-memory/plans/pending/` records what the first
  campaign already did — do not redo completed waves, close their residue.

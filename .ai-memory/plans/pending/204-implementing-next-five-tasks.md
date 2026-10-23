# Plan 204: Implementing Next Five Tasks (Steps 231–235)
Status: IN_PROGRESS
Sequence: 204
Total Steps in Batch: 5
Remaining Backlog after Batch: 15

## Subtasks
1. `01-pipeline-request-header-sanitizer.md`: Pipeline Request Header Sanitizer & Gemini Header Protocol (`Imp-645`) — `IN_PROGRESS`
2. `02-strictly-relative-git-paths-guard.md`: Strictly Relative Git Paths & Anti-Absolute-Path Guard (`Imp-646`) — `IN_PROGRESS`
3. `03-transient-sandbox-lifecycle-verifier.md`: Transient Sandbox Lifecycle & Zero-Remnants Verifier (`Imp-647`) — `IN_PROGRESS`
4. `04-rollback-reference-drift-verifier.md`: Rollback Reference & Content Drift Verifier (`Imp-648`) — `IN_PROGRESS`
5. `05-multi-stage-launcher-dispatcher.md`: Multi-Stage Launcher Fallback & Batch File Guard (`Imp-649`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

# Plan 212: Implementing Next Five Tasks (Steps 271–275)
Status: IN_PROGRESS
Sequence: 212
Total Steps in Batch: 5
Remaining Backlog after Batch: 0

## Subtasks
1. `01-pipeline-response-header-sanitizer.md`: Pipeline Response Header Sanitizer (`Imp-685`) — `IN_PROGRESS`
2. `02-window-control-capsule-formatter.md`: Window Control Capsule Formatter (`Imp-686`) — `IN_PROGRESS`
3. `03-fifo-dead-letter-queue-spillover-guard.md`: FIFO Dead Letter Queue Spillover Guard (`Imp-687`) — `IN_PROGRESS`
4. `04-caller-process-ancestry-tree-scanner.md`: Caller Process Ancestry Tree Scanner (`Imp-688`) — `IN_PROGRESS`
5. `05-github-release-notes-parity-guard.md`: GitHub Release Notes Parity Guard (`Imp-689`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

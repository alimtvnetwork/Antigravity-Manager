# Plan 211: Implementing Next Five Tasks (Steps 266–270)
Status: IN_PROGRESS
Sequence: 211
Total Steps in Batch: 5
Remaining Backlog after Batch: 0

## Subtasks
1. `01-windows-shell-dispatch-argument-escaper.md`: Windows Shell Dispatch Argument Escaper (`Imp-680`) — `IN_PROGRESS`
2. `02-fifo-queue-drain-monotonicity-verifier.md`: FIFO Queue Drain Monotonicity Verifier (`Imp-681`) — `IN_PROGRESS`
3. `03-streaming-chunk-boundary-guard.md`: Streaming Chunk Boundary Guard (`Imp-682`) — `IN_PROGRESS`
4. `04-truncation-omission-banner-formatter.md`: Truncation Omission Banner Formatter (`Imp-683`) — `IN_PROGRESS`
5. `05-individually-revertable-commit-guard.md`: Individually Revertable Commit Guard (`Imp-684`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

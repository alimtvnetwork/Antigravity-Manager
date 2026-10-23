# Plan 198: Implementing Next Five Tasks (Steps 206–210)
Status: IN PROGRESS (TAKEN)
Sequence: 198
Total Steps in Batch: 5
Remaining Backlog after Batch: 45

## Subtasks
1. `01-transport-backoff-stepper.md`: Transport Backoff Stepper with Jitter (`Imp-620`)
2. `02-transient-sandbox-manager.md`: Transient Sandbox Manager & Zero-Orphan Cleanup (`Imp-621`)
3. `03-fifo-prompt-queue-coordinator.md`: FIFO Prompt Queue Coordinator (`Imp-622`)
4. `04-account-switch-audit-emitter.md`: Account Switch Audit Emitter (`Imp-623`)
5. `05-rejection-breakdown-generator.md`: Rejection File-by-File Breakdown Generator (`Imp-624`)

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

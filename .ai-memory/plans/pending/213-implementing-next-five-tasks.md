# Plan 213: Implementing Next Five Tasks (Steps 276–280)
Status: IN_PROGRESS
Sequence: 213
Total Steps in Batch: 5
Remaining Backlog after Batch: 0

## Subtasks
1. `01-streaming-event-frame-parser.md`: Streaming Event Frame Parser (`Imp-690`) — `IN_PROGRESS`
2. `02-inline-select-dropdown-styler.md`: Inline Select Dropdown Styler (`Imp-691`) — `IN_PROGRESS`
3. `03-fifo-queue-compaction-guard.md`: FIFO Queue Compaction Guard (`Imp-692`) — `IN_PROGRESS`
4. `04-caller-ide-exit-code-enforcer.md`: Caller IDE Exit Code 2 Enforcer (`Imp-693`) — `IN_PROGRESS`
5. `05-transient-test-sandbox-lifecycle-guard.md`: Transient Test Sandbox Lifecycle Guard (`Imp-694`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).
- Master E2E test plan documented in `02-spec/21-app/213-implementing-next-five-tasks/e2e_master_plan.md`.

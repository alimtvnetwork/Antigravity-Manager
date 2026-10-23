# Plan 214: Implementing Next Five Tasks (Steps 281–285)
Status: IN_PROGRESS
Sequence: 214
Total Steps in Batch: 5
Remaining Backlog after Batch: 0

## Subtasks
1. `01-streaming-payload-accumulator.md`: Streaming Payload Accumulator (`Imp-695`) — `IN_PROGRESS`
2. `02-capsule-segmented-button-group-styler.md`: Capsule Segmented Button Group Styler (`Imp-696`) — `IN_PROGRESS`
3. `03-fifo-prompt-restore-cursor-stepper.md`: FIFO Prompt Restore Cursor Stepper (`Imp-697`) — `IN_PROGRESS`
4. `04-batch-file-execution-interceptor.md`: Batch File Execution Interceptor (`Imp-698`) — `IN_PROGRESS`
5. `05-contributor-authorship-preservation-guard.md`: Contributor Authorship Preservation Guard (`Imp-699`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).
- Master E2E test plan documented in `02-spec/21-app/214-implementing-next-five-tasks/e2e_master_plan.md`.

# Plan 205: Implementing Next Five Tasks (Steps 236–240)
Status: IN_PROGRESS
Sequence: 205
Total Steps in Batch: 5
Remaining Backlog after Batch: 10

## Subtasks
1. `01-thinking-budget-headroom-evaluator.md`: Pipeline Thinking Budget Headroom Evaluator (`Imp-650`) — `IN_PROGRESS`
2. `02-preflight-clippy-gate.md`: Cargo Clippy All-Features Preflight Gate (`Imp-651`) — `IN_PROGRESS`
3. `03-pr-unverified-paths-linter.md`: PR Unverified Paths & Cost Disclosure Linter (`Imp-652`) — `IN_PROGRESS`
4. `04-multi-stage-launcher-engine.md`: Multi-Stage Launcher Engine & Execution Ladder (`Imp-653`) — `IN_PROGRESS`
5. `05-fifo-prompt-order-enforcer.md`: Strict FIFO Prompt Order Enforcer & SQL Transformer (`Imp-654`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

# Plan 207: Implementing Next Five Tasks (Steps 246–250)
Status: IN_PROGRESS
Sequence: 207
Total Steps in Batch: 5
Remaining Backlog after Batch: 5

## Subtasks
1. `01-agent-prompt-sanitization-regex-engine.md`: Agent Prompt Sanitization Regex Engine (`Imp-660`) — `IN_PROGRESS`
2. `02-caller-ide-heartbeat-watchdog.md`: Caller IDE Vitality Heartbeat & Eviction Watchdog (`Imp-661`) — `IN_PROGRESS`
3. `03-changelog-tag-monotonic-gate.md`: Changelog Heading & Tag Monotonic Progression Gate (`Imp-662`) — `IN_PROGRESS`
4. `04-fifo-queue-index-analyzer.md`: FIFO Queue SQLite Composite Index Analyzer (`Imp-663`) — `IN_PROGRESS`
5. `05-bounded-matcher-timeout-verifier.md`: Bounded Matcher & Wait Timeout Verifier (`Imp-664`) — `IN_PROGRESS`

## Execution Invariants
- Zero builds and zero tests during implementation.
- Strictly relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors rule across Rust and TypeScript.
- Strict FIFO query ordering (`ORDER BY created_at ASC, id ASC`).

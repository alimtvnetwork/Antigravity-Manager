# End-to-End Master Execution Plan: Sequence 215

## 1. Batch Objective
Implement and verify updates across GitMap and Antigravity-Manager covering prompt backup and honest restoration on quota auto-switch, rerun randomized option auto-pick and persistence, GitMap code quality cleanup, and deferred end-to-end verification.

## 2. Work Breakdown & Subagent Allocations
- **Worker 01 (Task 1 & Task 2)**:
  - Repository audit of last 50-80 commits across AGM and GitMap.
  - AGM Backend: `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/repo_db/backup.rs` (incomplete-prompt sweep, honest outcome struct, re-attribution parity).
- **Worker 02 (Task 3 & Task 4)**:
  - AGM Frontend: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts`, `ConversationContextMenu.tsx` (rerun randomized default, 12+ prompt set, persistence).
  - GitMap Code Quality: drain completed pending plans in GitMap (`d:/work/gitmap/.ai-memory/plans/pending/`), eliminate boolean `== true` / `== false`, migrate `fmt.Errorf` to AppError.
- **Worker 03 (Task 5 & Master Plan Expansion)**:
  - Author and catalog exhaustive deferred E2E test scenarios (`E2E-215-01` through `E2E-215-05`) for account auto-switch prompt restore, quota depletion recovery, rerun auto-pick, and GitMap command execution.

## 3. Invariants
- Zero builds and zero tests during authoring.
- Exclusively relative Git paths.
- No silent errors rule.
- Strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).

# Migration and Verification Specification: Sequence 220

## Overview
- Sequence: 220
- Steps: 311–315
- Improvements: `Imp-725` through `Imp-729`
- Target Repositories: `Antigravity-Manager`, `gitmap`

## Verification Telemetry

### 1. GitMap PA Fleet Audit
- 68 repositories pulled and synchronized.
- GitMap commits audited and code quality modularized.

### 2. Prompt Restore Engine & Quota Exhaustion
- Implemented `reattribute_unrestored_prompt_backups` and `count_unrestored_prompts_for_instance`.
- Reattributed in-flight prompts during cross-instance profile rotation and quota exhaustion.
- Scoped restore fallback automatically claims and assigns orphaned records to the running instance.
- Unit tests in `tests_prompt_restore_honesty.rs` pass.

### 3. Rerun Randomizer UX & Persistent Execution
- Multi-round randomized picking prevents repetition across consecutive rounds.
- Round 1 dispatches immediately via `send_prompt_now`, Rounds 2..N enqueue sequentially into FIFO via `enqueue_prompt`.
- Unit tests in `rerunActions.test.ts` pass (9/9 frontend test files passed).
- Production build `npm run build` completed successfully.

### 4. GitMap Code Quality
- `cli/cmdfix/`: `fix_execute.go` (209 lines), `fix_execute_helpers.go` (198 lines).
- `cli/heatmap/`: `heatmap_cmd.go` (208 lines), `heatmap_cmd_flags.go` (219 lines).
- `cli/cmdcommit/`: `commit_push.go` (230 lines), `commit_push_stage.go` (211 lines), `commit_push_rewrite.go` (174 lines).
- All files strictly $\le 300$ lines with 8–15 line functions and structured error returns.
- Go unit tests in `cmdfix`, `heatmap`, `cmdcommit` passed with 0 failures.

### 5. Master E2E Integration Suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`)
- Gate 1: Inspect Running Prompts & Output Structure — PASS
- Gate 2: Seed Synthetic Multimodal Prompt with Attachment — PASS
- Gate 3: Backup Running Prompts with Multimodal Image Tracing — PASS
- Gate 4: Evict Active Prompt Row to Simulate Account Switch / Restore — PASS
- Gate 5: Execute Restore and Verify FIFO Order & Image Preservation — PASS
- Gate 6: Verify GitMap Command Delegation Parity — PASS
- Gate 7: Verify Caller IDE Session Protection — PASS
- Gate 8: Complete Cleanup of Test Records & Residual Artifacts — PASS
- Total: 25 | Passed: 25 | Failed: 0

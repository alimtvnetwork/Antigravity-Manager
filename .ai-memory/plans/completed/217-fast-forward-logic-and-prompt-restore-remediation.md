# Plan 217: Fast-Forward Logic Review & Prompt Restore Quota Recovery Remediation
Status: COMPLETED

## High Priority Non-Negotiable Instructions
1. Git pull BEFORE starting any work — completed via GitMap PA fleet sync.
2. Author architecture specification under `02-spec/21-app/217-fast-forward-logic-and-prompt-restore-remediation/01-architecture-spec.md`.
3. Search codebase exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
4. Strictly use relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Zero absolute paths.
5. Strictly ZERO builds (`cargo build`, `npm run build`, `vite build`) or test runs during active code modification; execute builds and test cases only after all improvements are finalized.
6. Enforce strict First-In-First-Out (FIFO) database ordering across SQLite queries (`ORDER BY created_at ASC, id ASC`).
7. Enforce No Silent Errors: all fallible paths propagate via `AppError` or record using `crate::error::record_ignored` with a `// Justification:` comment.

## Implementation Tasks

- [x] **Subtask 01**: Fast-Forward Rotation & Quota Fallback Harmonization
  - Eliminate instance hijacking in `src/stores/useInstanceStore/smart.ts:314-325`. Ensure `effectiveInstanceId` is strictly `instId`.
  - Harmonize `smartRotateProfileAccount` to delegate to `instanceService.fastForwardInstance(instId)` so UI and CLI execute the exact same unified backend pipeline.
  - In `src-tauri/src/modules/auto_switcher/selection.rs`, enable proportional sub-threshold fallback (`quota > 5.0`) when no accounts meet `quota > threshold`.
  - In `selection.rs`, fall back to cached quota when live Google API quota refresh times out or encounters network errors.
  - In `src-tauri/src/modules/auto_switcher/instances.rs`, restrict active in-use account locking to genuinely running instances (`is_running == true`).

- [x] **Subtask 02**: Incomplete Prompt Restore & CDP Resilience
  - In `src-tauri/src/modules/backup_prompts_db/restore.rs:351-353`, update `is_live_status_preserved(status)` to ONLY match `Some("completed")` so orphaned prompts from dead sessions/depleted quotas are restored and re-executed.
  - In `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs`, update test assertion to verify `!is_live_status_preserved(Some("running"))`.
  - In `src-tauri/src/modules/repo_db/backup.rs`, expand `is_all_or_default` to `(instance_id IN ('default', '__default__', '') OR instance_id IS NULL)`.
  - In `src-tauri/src/modules/repo_db/auto_resume.rs`, expand query to `status IN ('queued', 'pending', 'backed_up', 'running', 'in_flight', 'dispatched', 'blocked')`.
  - In `src-tauri/src/modules/instance/cdp_onboarding.rs:626`, add an adaptive retry ladder (4 attempts, 400ms sleep) in `submit_prompt_via_cdp_for_instance`.

- [x] **Subtask 03**: Rerun Randomizer Auto-Pick & Up-Next Panel Visibility
  - In `src/components/instances/PromptTreeView/verificationTemplates.ts`, expand `isNudge` check to recognize `"is it finished?"`, `"is it released?"`, `"is it completed?"`, `"finished?"`, `"released?"`, `"completed?"`, etc.
  - In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, ensure `RERUN_DEFAULT_MESSAGE_OPTION` is `'randomized'` and `loadPersistedRerunMessageOption()` reliably auto-picks from the built-in random set.
  - In `src-tauri/src/commands/cmd_running_up_next.rs`, expand `query_up_next` and `query_up_next_total` to `status IN ('queued', 'pending', 'backed_up')`.

- [x] **Subtask 04**: Final Verification & E2E Validation
  - Run Rust gate `cargo clippy --all-targets --all-features` and `cargo fmt`.
  - Run frontend gate `npm run build`.
  - Run targeted test suite to verify 100% green pass.

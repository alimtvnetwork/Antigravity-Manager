# Specification: Task 221 - GitMap and Antigravity Tools Manager Updates & Testing
## Component Specification

### 1. Account Auto-Switching & Prompt Restore Component (`Imp-731`)
- **Location**: `src-tauri/src/prompts/restore.rs` and `src-tauri/src/auto_switcher/rotation.rs`
- **Responsibilities**:
  1. `sweep_and_reattribute_orphaned_backups(pool: &SqlitePool, current_instance_id: &str) -> Result<usize, AppError>`:
     - Scans `prompt_backups` where `status = 'backed_up'` and `target_instance_id != current_instance_id`.
     - Reattributes uncompleted prompt backups to `current_instance_id` when an instance reconnects or recovers.
     - Preserves original `created_at` timestamp to guarantee strict FIFO sequence.
  2. `reconcile_interrupted_prompts(pool: &SqlitePool, target_instance_id: &str) -> Result<usize, AppError>`:
     - Identifies prompts in `active_prompts` where status is `pending` or `running` but the parent process died or was rotated away.
     - Creates backup records in `prompt_backups` with `status = 'backed_up'` and re-targets `target_instance_id`.
     - Logs recovery action via `tracing::info!`.

### 2. Rerun Randomizer & History Component (`Imp-732`)
- **Location**: `src/hooks/usePromptTreePromptActions.ts` and `src/components/prompts/DetailHeader.tsx`
- **Responsibilities**:
  1. `getRandomRerunNudge(history: string[] = []): string`:
     - Maintains a rich pool of varied questions:
       - "Is it finished?"
       - "Is it released?"
       - "Is it completed?"
       - "Are you confident?"
       - "Did all tests pass?"
       - "Are all requirements satisfied?"
       - "Is the verification green?"
       - "Are edge cases covered?"
     - Filters out any nudges present in `history.slice(-5)` (last 5 turns) to prevent repetitive user experience.
     - Returns a clean prompt string with iteration counter context if applicable.
  2. History persistence:
     - Stores recent nudge variations in local session state or audit recorder.

### 3. GitMap cmdpull Modularization Component (`Imp-733`)
- **Location**: `d:/work/gitmap/cli/cmdpull/`
- **Files to Refactor**:
  1. `pull_task_db.go` (586 lines) -> Decompose into:
     - `pull_task_db.go`: Core task database lifecycle and coordinator ($\le 280$ lines).
     - `pull_task_db_ops.go`: Database operations, status updates, and transaction handling ($\le 280$ lines).
     - `pull_task_db_query.go`: Query execution and data mapper helpers ($\le 280$ lines).
  2. `push.go` (353 lines) -> Decompose into:
     - `push.go`: CLI push entry point and flag validation ($\le 220$ lines).
     - `push_exec.go`: Remote push execution, retry logic, and branch handling ($\le 220$ lines).
- **Standards**:
  - Functions 8–15 lines.
  - Return `*apperror.AppError` directly.
  - Value semantics for config and small payloads.

### 4. Master Deferred E2E Verification Component (`Imp-734`)
- **Location**: `scripts/test-agm-gitmap-running-backup-e2e.ps1`
- **Additions**:
  - Gate 26: Reconnect prompt sweep and orphaned backup reattribution verification.
  - Gate 27: Multi-turn rerun diversity test (verifying 5 consecutive reruns produce 5 distinct variations).
  - Gate 28: GitMap `cmdpull` file length verification ($\le 300$ lines across all files).

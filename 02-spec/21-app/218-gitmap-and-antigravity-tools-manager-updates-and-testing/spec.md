# Specification: GitMap and Antigravity Tools Manager Updates and Testing (Sequence 218)
Sequence: 218
Improvements: `Imp-715` through `Imp-719`
Slug: `218-gitmap-and-antigravity-tools-manager-updates-and-testing`

## 1. Executive Summary
This specification defines the architectural enhancements, reliability fixes, and verification protocols for Sequence 218 across GitMap and Antigravity-Manager:
1. `Imp-715` (Step 301): **GitMap PA Fleet Status Audit & Coding Guidelines Ingestion**
   - Fleet pull status verification across all 68 repositories (clean and synchronized).
   - Ingestion of recent coding guidelines updates: installer script signal handling fixes and Python boolean naming conversions.
   - Comprehensive audit report in `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md`.
2. `Imp-716` (Step 302): **Account Auto-Switching & Prompt Restore Resiliency (Deep Engine Fix)**
   - Core user priority: Eliminate dropped prompts when account quota finishes and pending prompts get closed.
   - Root Cause 1 Remediation: Fix `stop_prompt_goal_workers_for_instance` in `src-tauri/src/modules/repo_db/goal_heartbeat.rs` so terminated/stopped workers never mark active prompts as fake `'completed'`; transition them to `'backed_up'` so the restore engine discovers and recovers them.
   - Root Cause 2 Remediation: Fix `inspect_conversation_transcript` in `src-tauri/src/modules/repo_db/transcript.rs` so turns ending in `"error"`, 429, `RESOURCE_EXHAUSTED`, or quota depletion are NOT classified as successful completion (`is_terminal_done`). Flag them as needing restore, and expand the discovery freshness window in `discovery.rs` from 60 seconds to 30 minutes.
   - Sweeps all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending`, `queued`, `blocked`, and quota-interrupted) to `backed_up` upon quota exhaustion or auto-switch.
   - Re-attributes unrestored prompt records in SQLite (`active_prompts` and `prompt_backups`) to the destination instance ID.
   - Unscoped FIFO fallback if destination instance has zero scoped backup records.
   - Honest `RestoreOutcome` reporting with explicit counts `{ restored_count, dispatched_count, failed_count, skipped_live, last_error }` without fake success.
3. `Imp-717` (Step 303): **Rerun Randomizer UX & Persistent Execution Engine**
   - Default `rerunMessageOption` auto-picked as `'randomized'` across DetailHeader, right-click context menu, and modal strips.
   - Expanded 25+ actionable variation bank (*"Is it finished?"*, *"Is it released?"*, *"Is it completed?"*, *"Please verify and finish the remaining items."*, *"Did all tests pass and is everything green?"*, etc.).
   - Clean prompt extraction: Losslessly extract original user prompt from conversation turns, stripping recursive header templates (`<<<ORIGINAL_TASK_PROMPT_START>>>`) without clearing the prompt.
   - Persistent `localStorage` settings under `agm:prompt-tree:rerun-option` and `agm:prompt-tree:rerun-count`.
4. `Imp-718` (Step 304): **GitMap Code Quality & CG Execute Suite**
   - Enforce coding standards across `gitmap` and `src-tauri`: zero boolean equality checks (`== true` / `== false`), structured `AppError` handling, bounded functions (8–15 lines), and clean pending plans drain.
5. `Imp-719` (Step 305): **Master Deferred E2E Plan & Final Live Testing Execution**
   - Author and verify comprehensive E2E test specifications `E2E-218-01` through `E2E-218-05`.
   - Strict Zero-Build and Zero-Test discipline during code authoring, followed by comprehensive live test execution upon completion of all improvements.

---

## 2. Invariants & Rules Enforced
- **Strictly Relative Git Paths**: All file paths and references must be repository-relative (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Zero Windows drive letters (`D:\`, `C:\`) or root paths.
- **Strict FIFO Ordering**: Every database query across prompt databases must strictly enforce `ORDER BY created_at ASC, id ASC`.
- **No Silent Errors Rule**: Swallowing an error without routing to `AppError` or `useErrorStore.getState().trackWarning` is strictly forbidden.
- **Zero-Build & Zero-Test Policy During Coding**: Strictly zero build and test runs during code authoring; full live verification runs only after all improvements are complete.
- **No Fake Success Invariant**: A task or prompt that failed or was interrupted must NEVER be marked as `'completed'`.

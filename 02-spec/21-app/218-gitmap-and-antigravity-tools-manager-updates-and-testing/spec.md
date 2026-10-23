# Specification: GitMap and Antigravity Tools Manager Updates and Testing (Sequence 218)
Sequence: 218
Improvements: `Imp-715` through `Imp-719`
Slug: `218-gitmap-and-antigravity-tools-manager-updates-and-testing`

## 1. Executive Summary
This specification defines the architectural enhancements, reliability fixes, and verification protocols for Sequence 218 across GitMap and Antigravity-Manager:
1. `Imp-715` (Step 301): **GitMap PA Fleet Status Audit & Coding Guidelines Ingestion**
   - Fleet pull status verification across all 68 repositories (2 active: `coding-guidelines-v24` and `repo-cache`, 66 up-to-date).
   - Ingestion of recent coding guidelines updates: installer script signal handling fixes and Python boolean naming conversions.
   - Comprehensive audit report in `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md`.
2. `Imp-716` (Step 302): **Account Auto-Switching & Prompt Restore Resiliency (Deep Engine Fix)**
   - Core user priority: Eliminate dropped prompts when account quota finishes and pending prompts get closed.
   - Sweeps all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending` not marked `completed` or `succeeded`) to `backed_up` upon quota exhaustion or auto-switch.
   - Re-attributes unrestored prompt records in SQLite (`active_prompts` and `prompt_backups`) to the destination instance ID.
   - Unscoped FIFO fallback if destination instance has zero scoped backup records.
   - Honest `RestoreOutcome` reporting with explicit counts `{ restored_count, dispatched_count, failed_count, skipped_live, last_error }` without fake success.
3. `Imp-717` (Step 303): **Rerun Randomizer UX & Persistent Execution Engine**
   - Default `rerunMessageOption` auto-picked as `'randomized'` across DetailHeader, right-click context menu, and modal strips.
   - Default rerun count elevated to 4 (range 1–8).
   - Expanded 16+ actionable variation bank (*"Is it finished?"*, *"Is it released?"*, *"Run comprehensive verification and report status"*, etc.).
   - Persistent `localStorage` settings under `agm:prompt-tree:rerun-option` and `agm:prompt-tree:rerun-count`.
4. `Imp-718` (Step 304): **GitMap Code Quality & CG Execute Suite**
   - Enforce coding standards across `d:/work/gitmap/`: zero boolean equality checks (`== true` / `== false`), structured `AppError` handling, function size limits, and clean pending plans drain.
5. `Imp-719` (Step 305): **Master Deferred E2E Plan & Final Live Testing Execution**
   - Author comprehensive E2E test specifications `E2E-218-01` through `E2E-218-05`.
   - Strict Zero-Build and Zero-Test discipline during code edits, followed by comprehensive live test execution upon completion of all improvements.

---

## 2. Invariants & Rules Enforced
- **Strictly Relative Git Paths**: All file paths and references must be repository-relative (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Zero Windows drive letters (`D:\`, `C:\`) or root paths.
- **Strict FIFO Ordering**: Every database query across prompt databases must strictly enforce `ORDER BY created_at ASC, id ASC`.
- **No Silent Errors Rule**: Swallowing an error without routing to `AppError` or `useErrorStore.getState().trackWarning` is strictly forbidden.
- **Zero-Build & Zero-Test Policy During Coding**: Strictly zero build and test runs during code authoring; full live verification runs only after all improvements are complete.

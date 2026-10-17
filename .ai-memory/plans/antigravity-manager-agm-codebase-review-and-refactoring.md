# Master Plan: AGM Codebase Review, E9001 Resolution, Throttling & UI Modernization

**Application:** Antigravity Manager (AGM)
**Task ID:** `antigravity-manager-agm-codebase-review-and-refactoring`
**Spec Links:**
- [01-architecture-spec.md](02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)
- [02-component-spec.md](02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-spec.md)

---

## Subtasks Breakdown

### Subtask 01: Backend E9001 Root-Cause Fix & Throttling Hygiene
**File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/01-backend-e9001-and-throttling.md`
**Owner:** Worker 01
**Status:** Completed
**Scope:**
1. In `src-tauri/src/modules/auto_switcher/instances.rs`:
   - In `is_instance_binding_stale_or_exhausted`: return `false` if `is_running == true`.
2. In `src-tauri/src/modules/auto_switcher/manual.rs`:
   - Strictly gather all accounts bound to running sibling instances (including default instance active account).
3. In `src-tauri/src/modules/auto_switcher/selection.rs`:
   - Check both `bound_account_id`, `bound_email`, and active default account against running sibling instances in Section 1 and Section 2.
   - Guard `select_and_verify_next_best_profile` against selecting running sibling instance accounts.
4. In `src-tauri/src/modules/repo_db/queue.rs`:
   - Check `PathBuf::from(&prompt.repo_path).exists()`. If non-existent, mark prompt as `failed` to break the 5-second infinite loop.
   - Update `updated_at = now` on dispatch error to enforce retry delay.
5. In `src-tauri/src/modules/account/quota_fetch.rs`:
   - Implement adaptive L1 quota cache TTL: 2m (<20%), 3m (<50%), 5m (>=50%/100%).
6. In `src-tauri/src/modules/supabase_sync/fleet.rs`:
   - Enforce 5-minute backoff on Supabase error.

### Subtask 02: Frontend Accounts Table UI Modernization & Tree Reset
**File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/02-frontend-accounts-ui-and-tree.md`
**Owner:** Worker 02
**Status:** Completed
**Scope:**
1. In `src/components/accounts/AccountTable/SortableAccountRow.tsx`:
   - Remove blinding `hover:!bg-white dark:hover:!bg-white hover:!text-black dark:hover:!text-black`.
   - Apply subtle dark-glass hover: `hover:bg-slate-100/70 dark:hover:bg-slate-800/50`.
2. In `src/components/accounts/AccountTable/AccountRowContent.tsx`:
   - Remove cell `group-hover:!bg-white` and `group-hover:!text-black`.
   - Ensure `Default Copy` badge has crisp high-contrast border.
3. In `src/components/accounts/AccountTable/cells/ActionsCell.tsx`:
   - Remove cell `group-hover:!bg-white`.
   - Reorder buttons: Fast-Forward (`FastForward`), Refresh (`RefreshCw`), Switch (`ArrowRightLeft`), Delete (`Trash2`), More (`MoreHorizontal`).
   - Move Tag icon into More menu.
   - Apply 3D Riseup yellow gradient to Switch button.
   - Flatten Delete and Switch when row is idle, highlighting on hover.
4. In `src/components/instances/PromptTreeView/TreePanel.tsx`:
   - Add working "Reset Filters" button when search/filters are active.
5. In `src-tauri/src/modules/delegate_updater/run.rs`:
   - Elevate console updater font: `font_family = 54`, `dw_font_size.y = 18`, Consolas font.

---

## Execution Waves

| Wave | Workers | Focus | Status |
|---|---|---|---|
| **Wave 1** | Worker 01 (Backend) & Worker 02 (Frontend & System) | E9001 fix, queue throttling, Accounts UI hover, 3D switch button, terminal updater font, tree filter reset | **PASSED** |
| **Wave 2** | Lead Orchestrator | Quality verification (`cargo fmt`, `cargo clippy`, `npm run build`), release bump (`4.226.0`), and GitMap verification | **PASSED** |

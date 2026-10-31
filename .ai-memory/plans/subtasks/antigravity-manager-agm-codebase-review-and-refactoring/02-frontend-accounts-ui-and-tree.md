# Subtask 02: Frontend Accounts Table UI Modernization, Tree Reset & Console Font

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`
**Owner:** Worker 02
**Status:** In Progress

---

## 1. Owned Files
- `src/components/accounts/AccountTable/SortableAccountRow.tsx`
- `src/components/accounts/AccountTable/AccountRowContent.tsx`
- `src/components/accounts/AccountTable/cells/ActionsCell.tsx`
- `src/components/instances/PromptTreeView/TreePanel.tsx`
- `src-tauri/src/modules/delegate_updater/run.rs`

---

## 2. Implementation Steps
1. **`src/components/accounts/AccountTable/SortableAccountRow.tsx`**:
   - Strip all `hover:!bg-white dark:hover:!bg-white hover:!text-black dark:hover:!text-black` and `group-hover:!bg-white dark:group-hover:!bg-white`.
   - Apply subtle dark-glass hover background: `hover:bg-slate-100/70 dark:hover:bg-slate-800/50`.
2. **`src/components/accounts/AccountTable/AccountRowContent.tsx`**:
   - Strip all `group-hover:!bg-white` and `group-hover:!text-black`.
   - Ensure `boundInstance` badge has crisp high-contrast border.
3. **`src/components/accounts/AccountTable/cells/ActionsCell.tsx`**:
   - Strip `group-hover:!bg-white dark:group-hover:!bg-white`.
   - Reorder action buttons:
     1. Fast-Forward button (`FastForward`, `>>`)
     2. Refresh button (`RefreshCw`, `↻`)
     3. Switch button (`ArrowRightLeft`, ⇄)
     4. Delete button (`Trash2`, 🗑️)
     5. More button (`MoreHorizontal`, ···)
   - Move Tag button into More menu dropdown.
   - Apply 3D Riseup yellow gradient to Switch button:
     `bg-gradient-to-b from-[#FDE047] via-[#FACC15] to-[#EAB308] text-slate-950 font-bold border border-[#CA8A04] shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)] active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]`
   - Flatten Delete and Switch when row is idle; highlight on hover.
4. **`src/components/instances/PromptTreeView/TreePanel.tsx`**:
   - Add working Reset Filters button when filters are active (`searchQuery`, `activeFilter`, `categoryFilter`, `selectedProjectScope`).
5. **`src-tauri/src/modules/delegate_updater/run.rs`**:
   - In `win32_console::setup_dedicated_updater_console()`:
     - Set `font_info.font_family = 54` (`FF_MODERN | FIXED_PITCH`).
     - Set `font_info.dw_font_size.y = 18`.
     - Set `font_info.font_weight = 600`.
     - Consolas with Lucida Console fallback.

---

## 3. Verification
- `npm run build` passes.
- `cargo check --lib` passes.

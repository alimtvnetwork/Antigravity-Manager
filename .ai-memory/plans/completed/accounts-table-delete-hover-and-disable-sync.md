# Completed Plan: Accounts Table Delete, Hover Inversion, Button Styling & Disable Flow

> **Task Slug:** `accounts-table-delete-hover-and-disable-sync`  
> **Status:** COMPLETED  
> **Completed At:** 2026-10-17  
> **Protocol:** V6 Continuous Loop (`execute-parent-task-with-n-steps-v6`)  
> **Specifications:**
> - [01-architecture-spec.md](../../02-spec/21-app/accounts-table-delete-hover-and-disable-sync/01-architecture-spec.md)
> - [02-ui-component-spec.md](../../02-spec/21-app/accounts-table-delete-hover-and-disable-sync/02-ui-component-spec.md)

---

## 1. User Request (Verbatim)

```text
The Delete button does not work yet, and I have given several times. It also needs to be fixed by selecting. If we select and click on Delete, it does not work as well. Make sure that you fix it. And when we hover over to an item, try to make that line as white and make the text as black. Invert the colors, but do not make those bold or anything, then the position will change. It looks bad. Also, make sure when we hover, the buttons should be highlighted right inside. The button role needs to be changed as well. And also, the switch button needs to be in a different yellow highlighter color, which should be different than others. Also, the Remove button should be red and white. The background would be red. The color would be white. Also add a disable option. So if we disable any account by selecting or by right-clicking on it, that disable item would not be selected by passport or the switching automatically. By default, user can try to switch. That's a different story. If you understood, then implement this and bump the minor version, and then release.
```

---

## 2. Visual Reference

![Accounts Table Screenshot](../../assets/screenshots/accounts-table-delete-hover-and-disable-sync-01.png)

---

## 3. Executive Summary of Implementations

### A. Root Cause Resolution for Delete Button (Single & Batch Selection)
- **Root Cause Identified:** `useAccountsHandlers.ts` correctly populated `deleteConfirmId` and `isBatchDelete`, but `src/pages/accounts/AccountsDialogs.tsx` completely lacked any `ModalDialog` rendering for deletion states. Consequently, clicking Delete in a row or selecting multiple rows and clicking "Delete Selected" never rendered a modal, and `executeDelete()` / `executeBatchDelete()` were never triggered.
- **Remediation:** Mounted destructive `ModalDialog` components in `AccountsDialogs.tsx` for both single-item deletion and batch deletion with confirmation summaries, cancel cleanup, and error-store tracking via `useErrorStore.getState().trackWarning`.

### B. Position-Locked Table Row Hover Inversion
- **Inverted Colors without Layout Shift:** In `SortableAccountRow.tsx` and `AccountRow.tsx`, rows invert to pure white background (`hover:!bg-white dark:hover:!bg-white`) and pure black text (`hover:!text-black dark:hover:!text-black`).
- **Position-Locking Invariant:** Retained static `font-medium` (never `hover:font-bold`), static `border-l-2`, and static cell padding. Character layout metrics and cell dimensions remain perfectly stable on hover without jitter.
- **Child Cells & Sticky Actions Alignment:** In `AccountRowContent.tsx` and `ActionsCell.tsx`, Quota cells and the sticky right-pinned actions cell explicitly invert (`group-hover:!bg-white dark:group-hover:!bg-white`) to form a contiguous white row.

### C. Action Buttons Color Scheme Overhaul
- **Yellow Highlighter Switch Button (`⇄`):** Styled in `ActionsCell.tsx` and `AccountCardActions.tsx` with high-visibility yellow highlighter styling (`bg-yellow-400 text-slate-900 border border-yellow-500 hover:bg-yellow-300 font-medium shadow-sm active:scale-95 rounded-[5px]`).
- **Solid Red Remove / Delete Button (`🗑️`):** Styled with solid red background and crisp white icon/text (`bg-red-600 text-white border border-red-700 hover:bg-red-700 hover:text-white shadow-sm active:scale-95 rounded-[5px]`) matching the user's reference screenshot.
- **Button Roles & Accessibility:** Added semantic `type="button"`, `role="button"`, and `aria-label` attributes to all action buttons in `ActionsCell.tsx`.

### D. Account Disable / Enable Lifecycle & Auto-Switch Exclusion
- **User Actions:**
  * Selection Toolbar: Added "Disable Selected" and "Enable Selected" action buttons (`<Ban className="w-3.5 h-3.5" />`) when accounts are selected (`selectedIds.size > 0`).
  * Action Menu: Added "Disable Account" / "Enable Account" toggle option inside the `...` more menu in `ActionsCell.tsx`.
- **Backend IPC & Storage:** Added `toggle_account_disabled` and `set_accounts_disabled` in `src-tauri/src/commands/cmd_accounts.rs` and `crud.rs`. Updates `account.disabled`, persists JSON, and synchronizes state.
- **Auto-Switcher Exclusion:** In `src-tauri/src/modules/auto_switcher/selection.rs` and `scoring.rs`, disabled accounts are 100% excluded from automated rotation, passport candidate scoring, and auto-switching (`score_candidate_account` returns `0.0`).
- **Manual User Switching:** Permissive manual switching is retained in `switch.rs`, allowing users to manually switch to their accounts if desired.

---

## 4. Verification Check Matrix

| Check | Target / Command | Result |
|---|---|---|
| Rust Unit Tests | `cargo test --lib modules::auto_switcher::scoring` | PASS (exit 0) |
| Rust Formatting | `cargo fmt -- --check` | PASS (exit 0, 0 diffs) |
| Rust Clippy | `cargo clippy --lib --all-features` | PASS (exit 0, 0 warnings) |
| Frontend Build | `npm run build` | PASS (exit 0, built in 18.06s) |
| Position Lock | Manual DOM inspection of `SortableAccountRow.tsx` | PASS (static border, no font-bold) |
| Modal Mounting | Inspection of `AccountsDialogs.tsx` | PASS (single & batch ModalDialog mounted) |
